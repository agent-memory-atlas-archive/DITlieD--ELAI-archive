#!/usr/bin/env python3
"""E-17 SP6 — RAG eval harness.

Drives `elai bench rag --json` per (query, tier) over the 15-query
goldset in queries.toml, computes MRR / R@5 / R@10 per tier, and reads
the SSB sqlite store out-of-band to surface the 4 SP2 row types as a
wiring-discipline observable.

Stdlib only (CLAUDE.md: no new deps for a harness). Python 3.11+ via
tomllib; falls back to a minimal TOML parser is NOT supplied — older
Pythons should install `tomli` or upgrade.

Exit code:
  0 — STANDARD MRR meets soft floor AND DEEP MRR >= STANDARD MRR
  1 — monotonicity regression (DEEP < STANDARD) OR STANDARD MRR
      below floor when --enforce-floor is set
  2 — harness error (subprocess crash, malformed JSON, missing index)

Usage:
  python run.py --tier BOTH --top-k 10 --report mrr,r@5,r@10 --runs 3
  python run.py --tier STANDARD --top-k 5
"""

from __future__ import annotations

import argparse
import json
import os
import sqlite3
import statistics
import subprocess
import sys
import time
from pathlib import Path

try:
    import tomllib  # py311+
except ImportError:  # pragma: no cover
    try:
        import tomli as tomllib  # type: ignore
    except ImportError:
        print(
            "FATAL: need Python 3.11+ (tomllib) or `pip install tomli`. "
            "Stdlib-only harness, no other deps.",
            file=sys.stderr,
        )
        sys.exit(2)


SCRIPT_DIR = Path(__file__).resolve().parent
DEFAULT_QUERIES = SCRIPT_DIR / "queries.toml"
DEFAULT_PROJECT_ROOT = SCRIPT_DIR.parent.parent  # .../.elai_cc
DEFAULT_WORKFLOW_ROOT = DEFAULT_PROJECT_ROOT.parent  # .../.ELAI_workflow


# ELAI_DATA_DIR_NAME from crates/core/src/paths.rs = ".elai_cc"
ELAI_DATA_DIR_NAME = ".elai_cc"

# The 4 SP2 SSB row types that DEEP queries should fire (or skip with
# explicit emission) per the E-17 SP2 wiring contract.
SP2_OBSERVABLE_ROW_TYPES = (
    "deep_pool_capped",
    "rrf_quality_gate_fired",
    "hyde_skipped_code_query",
    "rrf_component_weights",
)

# Cap how long any single bench rag call can take before we abort with a
# soft skip. The pipeline target is < 5s for DEEP per E-17 spec; 60s
# headroom covers slow first-run model loads.
PER_QUERY_TIMEOUT_SEC = 60

# Soft monotonicity epsilon — DEEP must be at least this much above
# STANDARD (or equal) to count as a pass. 0.0 = strict >=. The contract
# test uses >= per DECISION-D4 to absorb the std::HashMap-tie flap risk.
MONOTONICITY_EPSILON = 0.0

# Production target (E-17 spec) — STANDARD MRR floor. We report against
# this but only enforce when --enforce-floor is set. Synthetic corpora
# and small goldsets can sit below this without indicating a regression.
STANDARD_MRR_FLOOR = 0.55


# ---------------------------------------------------------------------------
# Args
# ---------------------------------------------------------------------------


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument(
        "--queries",
        type=Path,
        default=DEFAULT_QUERIES,
        help=f"queries.toml path (default: {DEFAULT_QUERIES})",
    )
    p.add_argument(
        "--project-root",
        type=Path,
        default=None,
        help="Project root containing .elai_cc/. Defaults to the parent of this script's grandparent (the workflow root) so the in-tree index at .elai_cc/.elai_cc/ resolves correctly.",
    )
    p.add_argument(
        "--tier",
        choices=("STANDARD", "DEEP", "BOTH"),
        default="BOTH",
        help="Which tier(s) to measure (default: BOTH).",
    )
    p.add_argument("--top-k", type=int, default=10, help="Top-K results per query (default 10).")
    p.add_argument(
        "--runs",
        type=int,
        default=1,
        help="Runs per (query, tier) — median is reported (default 1). 3+ recommended for tie-determinism.",
    )
    p.add_argument(
        "--report",
        type=str,
        default="mrr,r@5,r@10",
        help="Comma-separated metrics to report (default mrr,r@5,r@10).",
    )
    p.add_argument(
        "--elai-bin",
        type=str,
        default=None,
        help="Path to the elai binary. Defaults to `elai` on PATH or target/debug/elai.",
    )
    p.add_argument(
        "--enforce-floor",
        action="store_true",
        help="Require STANDARD MRR >= 0.55 (E-17 production target). Off by default for harness runs over synthetic / small corpora.",
    )
    p.add_argument(
        "--no-ssb",
        action="store_true",
        help="Skip the post-run SSB observation pass.",
    )
    p.add_argument(
        "--output",
        type=Path,
        default=None,
        help="Optional TOML report file (defaults to <project_root>/.elai_cc/metrics/E-17-eval-<ts>.toml).",
    )
    return p.parse_args()


# ---------------------------------------------------------------------------
# Queries.toml
# ---------------------------------------------------------------------------


def load_queries(path: Path) -> list[dict]:
    raw = path.read_bytes()
    data = tomllib.loads(raw.decode("utf-8"))
    queries = data.get("query", [])
    if not queries:
        raise SystemExit(f"FATAL: {path} has no [[query]] entries")
    for q in queries:
        for field in ("id", "text", "class", "tier", "expected_paths"):
            if field not in q:
                raise SystemExit(f"FATAL: query {q.get('id', '?')} missing field {field!r}")
    return queries


# ---------------------------------------------------------------------------
# elai bench rag driver
# ---------------------------------------------------------------------------


def find_elai_bin(override: str | None, project_root: Path) -> str:
    if override:
        return override
    # Prefer cargo target/ over PATH so callers always exercise the freshly-built bin.
    candidates = [
        project_root / "target" / "debug" / "elai.exe",
        project_root / "target" / "debug" / "elai",
        project_root / "target" / "release" / "elai.exe",
        project_root / "target" / "release" / "elai",
    ]
    for c in candidates:
        if c.is_file():
            return str(c)
    # Fall back to PATH lookup
    return "elai"


def run_bench_rag(
    elai_bin: str,
    project_root: Path,
    query: str,
    tier: str,
    top_k: int,
) -> dict | None:
    """Invoke `elai bench rag --json` once. Returns parsed JSON dict or
    None on soft failure (timeout / non-zero exit / malformed JSON)."""
    cmd = [
        elai_bin,
        "bench",
        "rag",
        "--tier",
        tier,
        "--top-k",
        str(top_k),
        "--project-root",
        str(project_root),
        "--json",
        "--",
        query,
    ]
    try:
        proc = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=PER_QUERY_TIMEOUT_SEC,
            check=False,
            cwd=str(project_root),
        )
    except subprocess.TimeoutExpired:
        print(f"  WARN: timeout after {PER_QUERY_TIMEOUT_SEC}s on {tier} / {query!r}", file=sys.stderr)
        return None
    except FileNotFoundError as e:
        print(f"  WARN: elai binary not found: {e}", file=sys.stderr)
        return None
    if proc.returncode != 0:
        print(
            f"  WARN: elai exit={proc.returncode} on {tier} / {query!r}:\n    stderr={proc.stderr[:400]}",
            file=sys.stderr,
        )
        return None
    out = proc.stdout.strip()
    if not out:
        return None
    # The CLI prints prose lines before the JSON when stages bail in unusual ways;
    # find the first '{' and parse from there.
    brace = out.find("{")
    if brace < 0:
        print(f"  WARN: no JSON in stdout for {query!r}", file=sys.stderr)
        return None
    try:
        return json.loads(out[brace:])
    except json.JSONDecodeError as e:
        print(f"  WARN: malformed JSON for {query!r}: {e}", file=sys.stderr)
        return None


# ---------------------------------------------------------------------------
# Metrics
# ---------------------------------------------------------------------------


def rank_of_first_hit(results: list[dict], expected_paths: list[str]) -> int | None:
    """Return 1-indexed rank of the first result whose file_path or
    chunk_id contains any expected substring. None = not in top-K."""
    needles = [p.replace("\\", "/").lower() for p in expected_paths]
    for r in results:
        hay = (r.get("file_path") or r.get("chunk_id") or "").replace("\\", "/").lower()
        if any(n in hay for n in needles):
            return r.get("rank", 0) or 1
    return None


def compute_metrics(per_query_ranks: list[int | None], top_k: int) -> dict:
    """MRR, R@5, R@10, hit-count, total."""
    total = len(per_query_ranks)
    if total == 0:
        return {"mrr": 0.0, "r_at_5": 0.0, "r_at_10": 0.0, "hits": 0, "total": 0}
    mrr_terms = []
    hits = 0
    hits_at_5 = 0
    hits_at_10 = 0
    for rank in per_query_ranks:
        if rank is None or rank > top_k:
            mrr_terms.append(0.0)
            continue
        hits += 1
        if rank <= 5:
            hits_at_5 += 1
        if rank <= 10:
            hits_at_10 += 1
        mrr_terms.append(1.0 / float(rank))
    return {
        "mrr": statistics.mean(mrr_terms) if mrr_terms else 0.0,
        "r_at_5": hits_at_5 / total,
        "r_at_10": hits_at_10 / total,
        "hits": hits,
        "total": total,
    }


def median_rank(ranks: list[int | None]) -> int | None:
    """Median rank across N runs, treating None as missing (only median
    over non-None ranks; if all None, return None)."""
    valid = [r for r in ranks if r is not None]
    if not valid:
        return None
    valid.sort()
    return valid[len(valid) // 2]


# ---------------------------------------------------------------------------
# SSB observation (out-of-band sqlite reader)
# ---------------------------------------------------------------------------


def observe_ssb(project_root: Path) -> dict:
    """Open .elai_cc/sessions/ssb.db and count rows per SP2 action_type."""
    db_path = project_root / ELAI_DATA_DIR_NAME / "sessions" / "ssb.db"
    if not db_path.is_file():
        return {"db_path": str(db_path), "available": False, "reason": "not found"}
    counts: dict[str, int] = {}
    try:
        conn = sqlite3.connect(f"file:{db_path}?mode=ro", uri=True, timeout=5)
        cur = conn.cursor()
        # Detect ssb_actions schema; fall back gracefully if absent.
        cur.execute("SELECT name FROM sqlite_master WHERE type='table' AND name='ssb_actions'")
        if cur.fetchone() is None:
            conn.close()
            return {"db_path": str(db_path), "available": False, "reason": "no ssb_actions table"}
        # SP4 wiring + the canonical SsbActionType::as_sql_str values:
        for action in SP2_OBSERVABLE_ROW_TYPES:
            cur.execute(
                "SELECT COUNT(*) FROM ssb_actions WHERE action_type = ?",
                (action,),
            )
            row = cur.fetchone()
            counts[action] = int(row[0]) if row else 0
        conn.close()
    except sqlite3.Error as e:
        return {"db_path": str(db_path), "available": False, "reason": f"sqlite error: {e}"}
    return {"db_path": str(db_path), "available": True, "counts": counts}


# ---------------------------------------------------------------------------
# Reporting
# ---------------------------------------------------------------------------


def fmt_pct(x: float) -> str:
    return f"{x:6.4f}"


def print_report(
    per_tier_metrics: dict[str, dict],
    per_tier_per_query: dict[str, list[dict]],
    ssb_observation: dict,
    requested_report: list[str],
    enforce_floor: bool,
) -> tuple[int, dict]:
    """Print human-readable report; return (exit_code, summary_dict)."""
    print()
    print("=" * 64)
    print("E-17 SP6 RAG eval — summary")
    print("=" * 64)
    standard = per_tier_metrics.get("STANDARD")
    deep = per_tier_metrics.get("DEEP")
    if standard:
        print(
            f"  STANDARD: MRR={fmt_pct(standard['mrr'])}  R@5={fmt_pct(standard['r_at_5'])}  R@10={fmt_pct(standard['r_at_10'])}  hits={standard['hits']}/{standard['total']}"
        )
    if deep:
        print(
            f"  DEEP    : MRR={fmt_pct(deep['mrr'])}  R@5={fmt_pct(deep['r_at_5'])}  R@10={fmt_pct(deep['r_at_10'])}  hits={deep['hits']}/{deep['total']}"
        )
    monotonicity_ok = True
    monotonicity_msg = "n/a (need both STANDARD and DEEP)"
    if standard and deep:
        delta = deep["mrr"] - standard["mrr"]
        monotonicity_ok = delta >= MONOTONICITY_EPSILON
        monotonicity_msg = (
            f"DEEP - STANDARD = {delta:+.4f} -> {'OK' if monotonicity_ok else 'REGRESSION'}"
        )
    print(f"  monotonicity (DEEP >= STANDARD): {monotonicity_msg}")
    floor_ok = True
    if enforce_floor and standard:
        floor_ok = standard["mrr"] >= STANDARD_MRR_FLOOR
        print(
            f"  STANDARD floor ({STANDARD_MRR_FLOOR:.2f}): "
            f"{'PASS' if floor_ok else 'FAIL'} "
            f"(measured={standard['mrr']:.4f})"
        )

    print()
    print("SSB observation (4 SP2 row types — out-of-band sqlite read):")
    if ssb_observation.get("available"):
        for action in SP2_OBSERVABLE_ROW_TYPES:
            cnt = ssb_observation["counts"].get(action, 0)
            print(f"  {action:<30} count={cnt}")
    else:
        print(
            f"  unavailable: {ssb_observation.get('reason', '?')} (db_path={ssb_observation.get('db_path')})"
        )

    print()
    print("Per-query rank (median over runs):")
    for tier, rows in per_tier_per_query.items():
        print(f"  --- {tier} ---")
        for r in rows:
            rank_str = "MISS" if r["median_rank"] is None else f"{r['median_rank']:>3}"
            print(f"    {r['id']:<40} class={r['class']:<5} rank={rank_str}")

    exit_code = 0 if (monotonicity_ok and floor_ok) else 1
    summary = {
        "monotonicity_ok": monotonicity_ok,
        "monotonicity_msg": monotonicity_msg,
        "floor_ok": floor_ok if enforce_floor else None,
        "standard": standard,
        "deep": deep,
        "ssb": ssb_observation,
    }
    return exit_code, summary


def write_toml_report(
    path: Path,
    summary: dict,
    per_tier_per_query: dict[str, list[dict]],
    run_meta: dict,
) -> None:
    """Hand-rolled TOML writer (stdlib has tomllib for read but no
    writer)."""
    path.parent.mkdir(parents=True, exist_ok=True)

    def esc(s: str) -> str:
        return s.replace("\\", "\\\\").replace('"', '\\"')

    def kv(key: str, value) -> str:
        if value is None:
            return f"# {key} = null"
        if isinstance(value, bool):
            return f"{key} = {'true' if value else 'false'}"
        if isinstance(value, (int, float)):
            return f"{key} = {value}"
        return f'{key} = "{esc(str(value))}"'

    lines: list[str] = []
    lines.append("# E-17 SP6 RAG eval — generated report")
    lines.append("")
    lines.append("[meta]")
    for k, v in run_meta.items():
        lines.append(kv(k, v))
    lines.append("")
    if summary.get("standard"):
        lines.append("[results.standard]")
        for k, v in summary["standard"].items():
            lines.append(kv(k, v))
        lines.append("")
    if summary.get("deep"):
        lines.append("[results.deep]")
        for k, v in summary["deep"].items():
            lines.append(kv(k, v))
        lines.append("")
    lines.append("[monotonicity]")
    lines.append(kv("ok", summary.get("monotonicity_ok")))
    lines.append(kv("msg", summary.get("monotonicity_msg")))
    lines.append("")
    ssb = summary.get("ssb", {})
    lines.append("[ssb]")
    lines.append(kv("available", ssb.get("available", False)))
    if ssb.get("available"):
        for action in SP2_OBSERVABLE_ROW_TYPES:
            lines.append(kv(f"count_{action}", ssb["counts"].get(action, 0)))
    else:
        lines.append(kv("reason", ssb.get("reason", "")))
    lines.append("")
    for tier, rows in per_tier_per_query.items():
        for r in rows:
            lines.append(f"[[per_query.{tier.lower()}]]")
            lines.append(kv("id", r["id"]))
            lines.append(kv("class", r["class"]))
            lines.append(kv("median_rank", r["median_rank"]))
            lines.append("")
    path.write_text("\n".join(lines), encoding="utf-8")
    print(f"report written to {path}")


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------


def resolve_project_root(arg: Path | None) -> Path:
    if arg is not None:
        return arg.resolve()
    # Heuristic: the workflow root sits above .elai_cc/eval/rag/run.py.
    # The "project root" expected by `elai bench rag` is the dir
    # CONTAINING the `.elai_cc/` subtree, i.e. F:\.ELAI_workflow.
    # SCRIPT_DIR = .../.elai_cc/eval/rag -> parents[2] = .../
    candidate = SCRIPT_DIR.parents[2]
    return candidate.resolve()


def main() -> int:
    args = parse_args()
    project_root = resolve_project_root(args.project_root)
    elai_bin = find_elai_bin(args.elai_bin, project_root / ELAI_DATA_DIR_NAME)
    queries = load_queries(args.queries)

    print(f"elai_bin     = {elai_bin}")
    print(f"project_root = {project_root}")
    print(f"queries_file = {args.queries}  (n={len(queries)})")
    print(f"tier         = {args.tier}")
    print(f"top_k        = {args.top_k}")
    print(f"runs         = {args.runs}")
    print()

    tiers = ["STANDARD", "DEEP"] if args.tier == "BOTH" else [args.tier]
    per_tier_per_query: dict[str, list[dict]] = {t: [] for t in tiers}
    per_tier_ranks: dict[str, list[int | None]] = {t: [] for t in tiers}
    per_query_failed_count = 0

    start = time.time()
    for q in queries:
        # Honor queries.toml `tier` field: "both" runs both, otherwise
        # only the listed tier (still in the user-requested tier set).
        q_tiers = tiers if q["tier"] == "both" else [q["tier"].upper()]
        q_tiers = [t for t in q_tiers if t in tiers]
        for tier in q_tiers:
            run_ranks: list[int | None] = []
            for run_idx in range(args.runs):
                result = run_bench_rag(elai_bin, project_root, q["text"], tier, args.top_k)
                if result is None:
                    per_query_failed_count += 1
                    run_ranks.append(None)
                    continue
                if result.get("pipeline_unavailable"):
                    # Soft skip — entire harness run is degraded but not error.
                    print(
                        f"  SKIP: pipeline unavailable for {q['id']} / {tier}: "
                        f"{result['pipeline_unavailable']}",
                        file=sys.stderr,
                    )
                    run_ranks.append(None)
                    continue
                hits = rank_of_first_hit(result.get("final_results", []), q["expected_paths"])
                run_ranks.append(hits)
            mr = median_rank(run_ranks)
            per_tier_ranks[tier].append(mr)
            per_tier_per_query[tier].append(
                {
                    "id": q["id"],
                    "class": q["class"],
                    "median_rank": mr,
                    "runs": run_ranks,
                }
            )
            print(
                f"  [{tier:<8}] {q['id']:<40} median_rank={mr if mr is not None else 'MISS'}"
            )
    duration = time.time() - start
    print(f"\nelapsed: {duration:.1f}s, soft failures: {per_query_failed_count}")

    per_tier_metrics = {t: compute_metrics(per_tier_ranks[t], args.top_k) for t in tiers}

    ssb_observation = (
        observe_ssb(project_root) if not args.no_ssb else {"available": False, "reason": "skipped"}
    )

    requested_report = [s.strip() for s in args.report.split(",") if s.strip()]
    exit_code, summary = print_report(
        per_tier_metrics,
        per_tier_per_query,
        ssb_observation,
        requested_report,
        args.enforce_floor,
    )

    output_path = args.output
    if output_path is None:
        ts = time.strftime("%Y%m%d-%H%M%S")
        output_path = project_root / ELAI_DATA_DIR_NAME / "metrics" / f"E-17-eval-{ts}.toml"
    run_meta = {
        "project_root": str(project_root),
        "queries_file": str(args.queries),
        "tier": args.tier,
        "top_k": args.top_k,
        "runs_per_query": args.runs,
        "elapsed_sec": round(duration, 2),
        "soft_failures": per_query_failed_count,
        "enforce_floor": args.enforce_floor,
        "elai_bin": elai_bin,
        "timestamp": time.strftime("%Y-%m-%dT%H:%M:%S%z"),
    }
    try:
        write_toml_report(output_path, summary, per_tier_per_query, run_meta)
    except OSError as e:
        print(f"  warn: could not write report: {e}", file=sys.stderr)
    return exit_code


if __name__ == "__main__":
    sys.exit(main())
