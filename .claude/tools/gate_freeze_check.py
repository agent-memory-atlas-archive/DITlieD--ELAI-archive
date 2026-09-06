#!/usr/bin/env python3
"""gate_freeze_check.py - freeze the wiring gate (rule R8, portable).

A run that can rewrite its own evaluator always writes itself to PASS (Berkeley
RDI, ImpossibleBench). This tool detects whether the frozen gate fileset listed
in gate-manifest.txt was modified, added, or deleted relative to a recorded
baseline (gate-manifest.lock). It is git-independent on purpose: in these
harnesses .claude/ is gitignored and some target workflows are not git repos, so
a `git diff` freeze cannot see harness files. Integrity is by content hash.

DO-333 traceability: the manifest now carries per-row trace fields
(objective_covered, technique, evidence_artifact, satisfaction) alongside the
path. --trace validates the schema and prints the traceability matrix. A
manifest with missing/invalid trace fields is a BLOCK (exit 3) on verify, lock,
and trace: locking an invalid schema is forbidden.

Hash: blake3 if the package is importable (preferred per the project standard),
otherwise stdlib hashlib.sha3_256 (strong; not the forbidden blake2/sha2). The
algorithm used is recorded per row in the lock file and must match on verify.

Usage:
  python gate_freeze_check.py --lock      # (re)record the baseline, out-of-band,
                                           # AFTER a deliberate human-reviewed gate
                                           # change. Prints what changed.
  python gate_freeze_check.py             # verify (default). Any modified / added /
                                           # deleted gate file -> BLOCK (exit 3).
  python gate_freeze_check.py --trace     # print the DO-333 traceability matrix,
                                           # validate schema, then verify (unless
                                           # --lock). Schema errors -> exit 3.
  Flags: --manifest PATH --lock-file PATH --root PATH --json

Exit: 0 all frozen files match baseline; 2 no baseline yet (run --lock); 3 BLOCK
(a frozen gate file was changed/added/deleted, the manifest is missing/empty,
or trace schema fields are missing/invalid).
"""
from __future__ import annotations

import argparse
import json
import os
import sys
from pathlib import Path
from typing import Dict, List, Optional, Tuple

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

try:
    import blake3 as _blake3  # type: ignore
    _ALGO = "blake3"
except Exception:  # noqa: BLE001 - any import failure falls back
    _blake3 = None
    import hashlib
    _ALGO = "sha3_256"

_VALID_TECHNIQUES = {"proof", "model-check", "abstract-interp", "RV", "test", "static"}
_VALID_EVIDENCE = {"SSB-row", "coverage", "proof", "hash"}
_VALID_SATISFACTION = {"REPLACE", "AUGMENT", "PARTIAL"}
_TRACE_FIELDS = ("objective_covered", "technique", "evidence_artifact", "satisfaction")


def _hash_file(path: str) -> str:
    if _blake3 is not None:
        h = _blake3.blake3()
    else:
        h = hashlib.sha3_256()  # type: ignore[name-defined]
    with open(path, "rb") as fh:
        for chunk in iter(lambda: fh.read(65536), b""):
            h.update(chunk)
    return h.hexdigest()


def _default_root() -> str:
    # script lives at <root>/.claude/tools/gate_freeze_check.py
    return str(Path(__file__).resolve().parents[2])


def _parse_manifest_rows(manifest: str) -> List[Dict]:
    """Parse the manifest into a list of row dicts.

    Each row dict has keys: path, objective_covered, technique,
    evidence_artifact, satisfaction. Rows with bare paths (no TAB or missing
    fields) are still collected (path only) so the caller can report them as
    schema errors.
    """
    out: List[Dict] = []
    try:
        with open(manifest, "r", encoding="utf-8") as fh:
            for raw in fh:
                ln = raw.strip()
                if not ln or ln.startswith("#"):
                    continue
                parts = ln.split("\t")
                row: Dict = {"path": parts[0].replace("\\", "/")}
                if len(parts) >= 5:
                    row["objective_covered"] = parts[1]
                    row["technique"] = parts[2]
                    row["evidence_artifact"] = parts[3]
                    row["satisfaction"] = parts[4]
                else:
                    row["objective_covered"] = ""
                    row["technique"] = ""
                    row["evidence_artifact"] = ""
                    row["satisfaction"] = ""
                out.append(row)
    except OSError:
        return []
    return out


def _read_manifest(manifest: str) -> List[str]:
    """Compatibility wrapper: return paths only (for run())."""
    return [r["path"] for r in _parse_manifest_rows(manifest)]


def _validate_trace_rows(rows: List[Dict]) -> List[str]:
    """Validate DO-333 trace fields. Returns a list of error strings."""
    errors: List[str] = []
    for row in rows:
        path = row.get("path", "?")
        for field in _TRACE_FIELDS:
            val = row.get(field, "")
            if not val:
                errors.append(f"{path}: missing {field}")
        if row.get("technique") and row["technique"] not in _VALID_TECHNIQUES:
            errors.append(f"{path}: invalid technique={row['technique']}")
        if row.get("evidence_artifact") and row["evidence_artifact"] not in _VALID_EVIDENCE:
            errors.append(f"{path}: invalid evidence_artifact={row['evidence_artifact']}")
        if row.get("satisfaction") and row["satisfaction"] not in _VALID_SATISFACTION:
            errors.append(f"{path}: invalid satisfaction={row['satisfaction']}")
    return errors


def _read_lock(lock_file: str) -> Dict[str, Tuple[str, str]]:
    """rel_path -> (algo, hexdigest)."""
    out: Dict[str, Tuple[str, str]] = {}
    try:
        with open(lock_file, "r", encoding="utf-8") as fh:
            for raw in fh:
                ln = raw.rstrip("\n")
                if not ln or ln.startswith("#"):
                    continue
                parts = ln.split("\t")
                if len(parts) == 3:
                    out[parts[0].replace("\\", "/")] = (parts[1], parts[2])
    except OSError:
        pass
    return out


def _write_lock(lock_file: str, rows: Dict[str, str]) -> None:
    lines = ["# gate-manifest.lock - frozen wiring-gate baseline (R8). Do NOT edit",
             "# by hand. Regenerate out-of-band with: gate_freeze_check.py --lock",
             "# format: relpath<TAB>algo<TAB>hexdigest"]
    for rel in sorted(rows):
        lines.append(f"{rel}\t{_ALGO}\t{rows[rel]}")
    Path(lock_file).write_text("\n".join(lines) + "\n", encoding="utf-8")


def _render_trace_matrix(rows: List[Dict]) -> str:
    lines = ["DO-333 TRACEABILITY MATRIX",
             "path\tobjective_covered\ttechnique\tevidence_artifact\tsatisfaction"]
    for row in rows:
        lines.append(
            f"{row['path']}\t{row.get('objective_covered', '')}\t"
            f"{row.get('technique', '')}\t{row.get('evidence_artifact', '')}\t"
            f"{row.get('satisfaction', '')}"
        )
    return "\n".join(lines)


def run(manifest: str, lock_file: str, root: str,
        do_lock: bool, do_trace: bool = False) -> Tuple[int, Dict]:
    full_rows = _parse_manifest_rows(manifest)
    if not full_rows:
        return 3, {"status": "BLOCK",
                   "reason": f"gate manifest missing or empty: {manifest}"}

    schema_errors = _validate_trace_rows(full_rows)
    if schema_errors:
        return 3, {
            "status": "BLOCK",
            "reason": "DO-333 trace schema errors in manifest",
            "schema_errors": schema_errors,
        }

    manifest_paths = [r["path"] for r in full_rows]

    current: Dict[str, Optional[str]] = {}
    missing_on_disk: List[str] = []
    for rel in manifest_paths:
        full = os.path.join(root, rel.replace("/", os.sep))
        if not os.path.isfile(full):
            missing_on_disk.append(rel)
            current[rel] = None
        else:
            current[rel] = _hash_file(full)

    if do_lock:
        if missing_on_disk:
            return 3, {"status": "BLOCK", "algo": _ALGO,
                       "reason": "cannot lock: frozen files missing on disk",
                       "missing": missing_on_disk}
        rows = {rel: h for rel, h in current.items() if h is not None}
        _write_lock(lock_file, rows)
        return 0, {"status": "LOCKED", "algo": _ALGO,
                   "locked_files": len(rows), "lock_file": lock_file}

    baseline = _read_lock(lock_file)
    if not baseline:
        return 2, {"status": "NO_BASELINE", "algo": _ALGO,
                   "reason": f"no baseline at {lock_file}; run --lock once "
                             f"out-of-band to record the frozen gate"}

    modified: List[str] = []
    added: List[str] = []      # in manifest, not in baseline (new, un-baselined)
    deleted = list(missing_on_disk)
    algo_mismatch: List[str] = []
    for rel in manifest_paths:
        cur = current[rel]
        if cur is None:
            continue  # already in deleted
        if rel not in baseline:
            added.append(rel)
            continue
        b_algo, b_hash = baseline[rel]
        if b_algo != _ALGO:
            algo_mismatch.append(f"{rel} (locked with {b_algo}, verifying with {_ALGO})")
            continue
        if cur != b_hash:
            modified.append(rel)

    tampered = bool(modified or added or deleted or algo_mismatch)
    result: Dict = {
        "status": "BLOCK" if tampered else "OK",
        "algo": _ALGO,
        "checked": len(manifest_paths),
        "modified": modified,
        "added_unbaselined": added,
        "deleted": deleted,
        "algo_mismatch": algo_mismatch,
    }
    if do_trace:
        result["trace_matrix"] = _render_trace_matrix(full_rows)
    if tampered:
        result["reason"] = ("frozen gate fileset changed during a run -> BLOCK. "
                            "The gate is authored out-of-band only (R8). If this "
                            "change is a deliberate, human-reviewed gate update, "
                            "re-lock with --lock; otherwise revert it.")
        return 3, result
    return 0, result


def render(r: Dict) -> str:
    o = [f"GATE FREEZE CHECK - {r['status']} (hash={r.get('algo','?')})"]
    if r.get("trace_matrix"):
        o.append(r["trace_matrix"])
    if "reason" in r:
        o.append(f"  reason: {r['reason']}")
    if r.get("schema_errors"):
        o.append("  schema_errors:")
        for e in r["schema_errors"]:
            o.append(f"    - {e}")
    for k in ("modified", "added_unbaselined", "deleted", "algo_mismatch", "missing"):
        if r.get(k):
            o.append(f"  {k}:")
            for x in r[k]:
                o.append(f"    - {x}")
    if r["status"] in ("OK", "LOCKED"):
        o.append(f"  checked/locked: {r.get('checked', r.get('locked_files', 0))} files")
    return "\n".join(o)


def main() -> int:
    here = Path(__file__).resolve().parent
    ap = argparse.ArgumentParser(description="Freeze the wiring gate (R8)")
    ap.add_argument("--manifest", default=str(here / ".." / "gate-manifest.txt"))
    ap.add_argument("--lock-file", default=str(here / ".." / "gate-manifest.lock"))
    ap.add_argument("--root", default=_default_root())
    ap.add_argument("--lock", action="store_true",
                    help="record the baseline (out-of-band, after a reviewed change)")
    ap.add_argument("--trace", action="store_true",
                    help="print the DO-333 traceability matrix and validate schema")
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args()
    manifest = str(Path(args.manifest).resolve())
    lock_file = str(Path(args.lock_file).resolve())
    code, result = run(manifest, lock_file, args.root, args.lock, args.trace)
    print(json.dumps(result, indent=2) if args.json else render(result))
    return code


if __name__ == "__main__":
    sys.exit(main())
