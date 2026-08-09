#!/usr/bin/env python3
"""Deterministic SOLID proxy checks; provisional defaults are replaced by measured values via --calibrate."""
import argparse, json, re, statistics, subprocess, sys
from pathlib import Path

# PROVISIONAL / observe-only: replace with ELAI measurements, not literature constants.
DEFAULT_THRESHOLDS = {"K_FANOUT": 10, "K_METHODS": 10, "K_LINES": 200, "K_TRAIT_METHODS": 10}
UNCOVERED = ["OCP", "LSP"]
REF_TAGS = {"CALL_RESOLVED", "SCIP_REF", "TYPE_REF"}
DEF_TAGS = {"DEF_STRUCT", "DEF_CLASS"}

def _row(line, number):
    fields = line.rstrip("\n").split("\t")
    if len(fields) < 6 or not fields[0] or not fields[1] or ":" not in fields[2]:
        raise ValueError(f"malformed map row {number}")
    return {"tag": fields[0], "symbol": fields[1], "location": fields[2], "extra": fields[4], "snippet": fields[5]}

def load_map(path):
    rows, errors = [], []
    try:
        with path.open(encoding="utf-8", errors="replace") as stream:
            for number, line in enumerate(stream, 1):
                if not line.strip() or line.startswith("#"):
                    continue
                try: rows.append(_row(line, number))
                except ValueError as exc: errors.append(str(exc))
    except OSError as exc:
        raise ValueError(f"cannot read map {path}: {exc}") from exc
    if errors:
        raise ValueError(f"unparseable map {path}: {len(errors)} malformed row(s); first: {errors[0]}")
    if not rows:
        raise ValueError(f"unparseable map {path}: no data rows")
    return rows

def _files(args):
    if args.files:
        paths = [Path(x.strip()).as_posix() for x in args.files.split(",") if x.strip()]
        if not paths: raise ValueError("--files resolved to an empty changeset")
        return set(paths)
    try:
        out = subprocess.run(["git", "diff", "--name-only", args.base, "--"], capture_output=True, text=True, check=False)
    except OSError as exc: raise ValueError(f"cannot derive changeset from git: {exc}") from exc
    if out.returncode or not out.stdout.strip():
        raise ValueError(f"empty changeset from --base {args.base}")
    return {Path(x).as_posix() for x in out.stdout.splitlines() if x.strip()}

def _map_path(raw):
    if raw: return Path(raw)
    here = Path.cwd().resolve()
    for parent in (here, *here.parents):
        candidate = parent / "elai.map"
        if candidate.is_file(): return candidate
    return Path.cwd() / "elai.map"

def _changed(row, files):
    return Path(row["location"].rsplit(":", 1)[0]).as_posix() in files

def _related(a, b):
    return a == b or a.startswith(b + "::") or a.startswith(b + ".")

_IDENTIFIER_RE = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")


def _contains_identifier(text, identifier):
    return identifier in _IDENTIFIER_RE.findall(text)


def _line_count(files, unreadable_files=None, root=None):
    if unreadable_files is None:
        unreadable_files = set()
    total = 0
    for name in files:
        resolved = (Path(root) / name) if root else Path(name)
        try:
            with resolved.open(encoding="utf-8", errors="replace") as stream:
                total += sum(1 for _ in stream)
        except OSError:
            unreadable_files.add(Path(name).as_posix())
    return total

def _finding(tag, severity, symbol, message, location=""):
    return {"tag": tag, "severity": severity, "symbol": symbol, "message": message, "location": location}

def scan(rows, files, thresholds, root=None):
    defs = [r for r in rows if r["tag"] in DEF_TAGS and _changed(r, files)]
    findings = []
    unreadable_files = set()
    for d in defs:
        sym = d["symbol"]
        fan = sum(1 for r in rows if r["tag"] in REF_TAGS and _related(r["symbol"], sym))
        methods = sum(1 for r in rows if r["tag"] == "DEF_METHOD" and _related(r["symbol"], sym))
        lines = _line_count({Path(d["location"].rsplit(":", 1)[0]).as_posix()}, unreadable_files, root)
        if fan > thresholds["K_FANOUT"] and (methods > thresholds["K_METHODS"] or lines > thresholds["K_LINES"]):
            findings.append(_finding("C-MAINTAINABILITY-SRP", "BLOCK", sym, f"god-object: fan_out={fan}, method_count={methods}, lines={lines}", d["location"]))
    # A concrete reference in a different crate from its definition is DIP-risk when
    # the map also records a trait implementation and a trait definition.
    traits = {r["symbol"] for r in rows if r["tag"] == "DEF_TRAIT"}
    impls = [r for r in rows if r["tag"] == "IMPL_TRAIT"]
    seen = set()
    for r in rows:
        if r["tag"] != "TYPE_REF" or not _changed(r, files): continue
        concrete = r["symbol"]
        matching = [i for i in impls if _contains_identifier(i["symbol"], concrete) or _contains_identifier(i["snippet"], concrete) or _contains_identifier(i["extra"], concrete)]
        if matching and any(_contains_identifier(i["symbol"], t) or _contains_identifier(i["snippet"], t) or _contains_identifier(i["extra"], t) for i in matching for t in traits):
            key = ("C-MAINTAINABILITY-DIP", concrete, Path(r["location"].rsplit(":", 1)[0]).as_posix())
            if key not in seen:
                seen.add(key)
                findings.append(_finding("C-MAINTAINABILITY-DIP", "WARN", concrete, "downstream concrete type reference to trait-covered upstream implementation", r["location"]))
    for trait in (r for r in rows if r["tag"] == "DEF_TRAIT" and _changed(r, files)):
        name = trait["symbol"]
        count = sum(1 for r in rows if r["tag"] == "DEF_METHOD" and _related(r["symbol"], name))
        if count <= thresholds["K_TRAIT_METHODS"]: continue
        implementors = [i for i in impls if _contains_identifier(i["symbol"], name) or _contains_identifier(i["snippet"], name) or _contains_identifier(i["extra"], name)]
        for imp in implementors:
            impl_name = imp["symbol"].split(":", 1)[0]
            used = sum(1 for r in rows if r["tag"] == "DEF_METHOD" and _related(r["symbol"], impl_name))
            if used * 2 < count:
                findings.append(_finding("C-MAINTAINABILITY-ISP", "WARN", name, f"trait methods={count}, implementor={impl_name} uses={used}", trait["location"]))
                break
    return findings, sorted(unreadable_files)

def _percentiles(values):
    if not values: return {"count": 0, "median": 0, "p90": 0, "p99": 0}
    vals = sorted(values)
    def pct(q): return vals[min(len(vals)-1, int((len(vals)-1)*q))]
    return {"count": len(vals), "median": statistics.median(vals), "p90": pct(.9), "p99": pct(.99)}

def calibrate(rows):
    symbols = {r["symbol"] for r in rows if r["tag"] in DEF_TAGS}
    fan = [sum(1 for r in rows if r["tag"] in REF_TAGS and _related(r["symbol"], s)) for s in symbols]
    methods = [sum(1 for r in rows if r["tag"] == "DEF_METHOD" and _related(r["symbol"], s)) for s in symbols]
    trait_names = {r["symbol"] for r in rows if r["tag"] == "DEF_TRAIT"}
    trait_methods = [sum(1 for r in rows if r["tag"] == "DEF_METHOD" and _related(r["symbol"], s)) for s in trait_names]
    return {"fan_out": _percentiles(fan), "method_count": _percentiles(methods), "trait_method_count": _percentiles(trait_methods)}

def main(argv=None):
    ap = argparse.ArgumentParser(description="SOLID proxy changeset checker")
    ap.add_argument("--files"); ap.add_argument("--base", default="HEAD"); ap.add_argument("--map", dest="map_path")
    ap.add_argument("--root"); ap.add_argument("--thresholds"); ap.add_argument("--calibrate", action="store_true"); ap.add_argument("--json", action="store_true")
    args = ap.parse_args(argv)
    try:
        path = _map_path(args.map_path); rows = load_map(path)
        if args.calibrate:
            payload = {"map": str(path), "calibration": calibrate(rows)}; print(json.dumps(payload, indent=2) if args.json else json.dumps(payload)); return 0
        files = _files(args)
        root = args.root
        if root is None:
            map_dir = path.parent.resolve()
            candidate = map_dir / ".elai_cc"
            root = str(candidate) if candidate.is_dir() else str(map_dir)
        thresholds = dict(DEFAULT_THRESHOLDS)
        if args.thresholds:
            with Path(args.thresholds).open(encoding="utf-8") as f: thresholds.update(json.load(f))
        findings, unreadable_files = scan(rows, files, thresholds, root)
        payload = {"verdict": "BLOCK" if any(f["severity"] == "BLOCK" for f in findings) else "PASS", "files": sorted(files), "findings": findings, "uncovered": UNCOVERED, "unreadable_files": unreadable_files, "map": str(path)}
        if args.json: print(json.dumps(payload, indent=2))
        else:
            print(f"{payload['verdict']}: {len(findings)} finding(s)")
            for f in findings: print(f"{f['severity']}: {f['tag']} {f['symbol']} -- {f['message']}")
        return 1 if payload["verdict"] == "BLOCK" else 0
    except (ValueError, OSError, json.JSONDecodeError, TypeError, KeyError) as exc:
        print(str(exc), file=sys.stderr); return 2

if __name__ == "__main__": sys.exit(main())
