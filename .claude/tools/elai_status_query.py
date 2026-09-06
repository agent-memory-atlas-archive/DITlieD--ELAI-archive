"""elai_status_query.py - instant wiring-status reads for LLMs (stdlib only).

Reads the elai-status.json snapshot the feeder/watcher maintain and answers
"what is wired, what is not" in one subsecond call - no agents, no checker
re-runs, no plan-file reads.

POSITIONING (Layer 0, advisory): this is a CACHE of the last verified state,
stamped with its age. It is a triage/pre-check layer for planning and
exploration. It is NEVER a done-gate: authoritative wiring verdicts for a
changeset still come from the live checkers (production_caller_check.py /
python_wire_check.py / wiring_contract_check.py) per wiring-enforcement.md.

Usage:
  python elai_status_query.py                    # compact summary (default)
  python elai_status_query.py --letter H         # one generation in detail
  python elai_status_query.py --punch            # the open punch list
  python elai_status_query.py --health           # dashboard health counts
  python elai_status_query.py --json             # machine slice of the above
  python elai_status_query.py --max-age-min 180  # exit 3 if snapshot older

Exit codes: 0 ok | 2 snapshot missing/unreadable | 3 stale beyond --max-age-min.
Works from Windows AND from WSL (pass --file /mnt/f/... or rely on the default
resolution relative to this script).
"""

from __future__ import annotations

import argparse
import json
import sys
import time
from pathlib import Path

DEFAULT = Path(__file__).resolve().parents[1].parent / "workflow" / "status" / "elai-status.json"


def _print_disposition(d) -> None:
    """Render a walk's terminal disposition under a punch item (a gap a walk
    already analyzed): no re-analysis needed, see the evidence ref."""
    if not d:
        return
    ref = f"; {d['evidence_ref']}" if d.get("evidence_ref") else ""
    print(f"      -> [{d.get('verdict', '?')}] {d.get('reason', '')}"
          f"  (walk {d.get('source_walk', '?')}{ref})")


def age_str(minutes: float) -> str:
    if minutes < 1:
        return "now"
    if minutes < 60:
        return f"{int(minutes)}m ago"
    if minutes < 48 * 60:
        return f"{minutes / 60:.0f}h ago"
    return f"{minutes / 1440:.0f}d ago"


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--file", default=str(DEFAULT))
    ap.add_argument("--letter")
    ap.add_argument("--punch", action="store_true")
    ap.add_argument("--health", action="store_true")
    ap.add_argument("--benchmarks", action="store_true")
    ap.add_argument("--spine", action="store_true")
    ap.add_argument("--rework", action="store_true")
    ap.add_argument("--json", action="store_true", dest="as_json")
    ap.add_argument("--max-age-min", type=float, default=None)
    args = ap.parse_args()

    path = Path(args.file)
    try:
        snap = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        print(f"SNAPSHOT MISSING: {path} (run elai_status_feeder.py; watcher normally maintains it)")
        return 2
    except Exception as exc:  # noqa: BLE001
        print(f"SNAPSHOT UNREADABLE: {type(exc).__name__}: {exc}")
        return 2

    age_min = max(0.0, (time.time() - snap.get("generated_at", 0)) / 60)
    stale = args.max_age_min is not None and age_min > args.max_age_min

    audit = snap.get("audit", {})
    tally = audit.get("tally", {}) if audit.get("ok") else {}
    corpus = snap.get("corpus", {}).get("letters", {}) if snap.get("corpus", {}).get("ok") else {}
    punch = audit.get("punch", []) if audit.get("ok") else []
    health = snap.get("health", {})
    allow = snap.get("allowlist", {})

    if args.as_json:
        out = {
            "advisory_cache": True,
            "snapshot_age_min": round(age_min, 1),
            "stale": stale,
            "generated_at_iso": snap.get("generated_at_iso"),
            "git_head": snap.get("git", {}).get("head"),
            "open_count": audit.get("open_count"),
            "punch": punch,
            "dispositions": snap.get("dispositions", {}).get("dispositions", {}),
            "tally": tally,
            "unaudited": {k: v for k, v in corpus.items() if k not in tally},
            "health": {k: health.get(k) for k in ("ok", "mount_counts", "ipc_counts", "error")},
            "registered_dark_flags": allow.get("registered_flags"),
            "benchmarks": snap.get("benchmarks", {}).get("rows"),
            "proof_spine": snap.get("proof_spine", {}).get("rows"),
            "ui_rework": snap.get("ui_rework", {}).get("rows"),
            "authoritative_note": "cache of last feeder run; changeset done-gates must run the live checkers",
        }
        if args.letter:
            L = args.letter.upper()
            out = {
                "advisory_cache": True, "snapshot_age_min": round(age_min, 1), "stale": stale,
                "letter": L, "tally": tally.get(L), "unaudited_files": corpus.get(L, {}).get("files") if L not in tally else None,
                "punch_for_letter": [p for p in punch if p.get("id", "").startswith(L)],
            }
        print(json.dumps(out, indent=1))
        return 3 if stale else 0

    hdr = f"ELAI STATUS (advisory cache) | snapshot {age_str(age_min)} | git {snap.get('git', {}).get('head', '--')}"
    if stale:
        hdr += f" | STALE (>{args.max_age_min:.0f}m)"
    print(hdr)

    if args.letter:
        L = args.letter.upper()
        row = tally.get(L)
        if row:
            print(f"{L}: {row['plans']} plans | {row['wired']} wired | {row['partial']} partial | "
                  f"{row['dark']} dark | {row['not_started']} not-started | {row['bench_gpu']} gpu | {row['other']} deferred")
        elif L in corpus:
            print(f"{L}: {corpus[L]['files']} files | UNAUDITED (no verdict yet, not counted)")
        else:
            print(f"{L}: unknown letter")
        for p in punch:
            if p.get("id", "").startswith(L):
                print(f"  OPEN {p['id']}: {p['desc']}")
                _print_disposition(p.get("disposition"))
    elif args.punch:
        print(f"open punch items: {audit.get('open_count', '?')} (audit source: {audit.get('source', '?')})")
        for p in punch:
            print(f"  {p['id']}: {p['desc']}")
            _print_disposition(p.get("disposition"))
    elif args.health:
        if not health.get("ok"):
            print(f"  HEALTH LEG ERROR: {health.get('error')}")
        for group in ("mount_counts", "ipc_counts"):
            for k, v in (health.get(group) or {}).items():
                print(f"  {k}: {v}")
    elif args.benchmarks:
        bm = snap.get("benchmarks", {})
        print(f"benchmarks: {bm.get('complete', '?')}/{bm.get('total', '?')} runnable done | "
              f"{bm.get('n_rows', '?')} registry rows | src: {bm.get('source', '?')}")
        for b in bm.get("rows", []):
            gate = f" gate:{b['gate']}" if b.get("gate") and b.get("gate") != "none" else ""
            ps = f" [{b['proof_spine']}]" if b.get("proof_spine") else ""
            art = "" if not b.get("artifact") else f"  {b['artifact']['path']}"
            print(f"  {b['name']:22} {b['state'].upper():10}{ps}{gate}{art}")
    elif args.spine:
        sp = snap.get("proof_spine", {})
        if not sp.get("ok", True) and sp.get("error"):
            print(f"  PROOF-SPINE LEG ERROR: {sp['error']}")
        else:
            print(f"proof spine: {sp.get('measured', '?')}/{sp.get('total', '?')} MEASURED "
                  f"(advisory; never a done-gate)")
            for r in sp.get("rows", []):
                lane = f"  lane[{r['lane_tag']}]" if r.get("lane_tag") else ""
                print(f"  {r['id']:5} {r['verdict']:15} gate:{r.get('gate')}  {r['name']}{lane}")
    elif args.rework:
        for r in snap.get("ui_rework", {}).get("rows", []):
            print(f"  {r['view']}: {r['state']}")
    else:
        wired = sum(r["wired"] for r in tally.values())
        plans = sum(r["plans"] for r in tally.values())
        unaudited = sum(v["files"] for k, v in corpus.items() if k not in tally)
        print(f"OPEN {audit.get('open_count', '?')} | lattice {wired}/{plans} audited units wired | "
              f"{unaudited} unaudited files (newer letters) | {allow.get('registered_flags', '?')} registered-dark flags")
        for L in sorted({**corpus, **tally}):
            row = tally.get(L)
            if row:
                print(f"  {L}: {row['wired']}/{row['plans']} wired | {row['partial']} partial | "
                      f"{row['dark'] + row['bench_gpu'] + row['other']} dark/parked | {row['not_started']} open")
            else:
                print(f"  {L}: {corpus[L]['files']} files | unaudited")
        if punch:
            print("open gaps: " + ", ".join(p["id"] for p in punch))
    print("note: cache of the last feeder run; a changeset done-gate still runs the live checkers")
    return 3 if stale else 0


if __name__ == "__main__":
    sys.exit(main())
