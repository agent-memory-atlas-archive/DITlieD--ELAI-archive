"""Truthful ELAI progress-core resolver.

The plan registry owns identity only.  Review settlement, implementation state,
and proof state are resolved independently from live ledgers.  In particular,
``drained`` means "review settled" and never means "verified delivered" unless
there is a reachable landed commit and durable gate evidence.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import sys
import time
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any

ELAI_ROOT = Path(__file__).resolve().parents[2]
TEIKOKU_ROOT = ELAI_ROOT.parent
REGISTRY_PATH = ELAI_ROOT / ".elai_cc" / "progress" / "plan-registry.json"
WALK_ROOT = TEIKOKU_ROOT / "conductor" / "opus_walk"
DISPOSITIONS_PATH = WALK_ROOT / "plan-dispositions.json"
WALK_STATE_PATH = WALK_ROOT / "walk-state.json"
PLAN_STATUS_DIR = WALK_ROOT / "plan-status"
AUDIT_PATH = WALK_ROOT / "evidence" / "drain-audit" / "DRAIN-AUDIT-2026-07-18.json"
CARD_INDEX_BUILDER = WALK_ROOT / "corpus_drain.py"
REGISTRY_SCHEMA_VERSION = 1
STATUS_SCHEMA_VERSION = 4
PLAN_ID_RE = re.compile(r"^([A-Q]-\d+(?:\.\d+)?[a-z]?(?:-[A-Za-z0-9]+)*)")
INDEX_DOC_RE = re.compile(r"^[A-Q]-0(?:_|$)")
ALIASES = {"H-28-WIRE-1": "H-28"}
SYNTHETIC_IDS = {"PS-1", "CODEX-DEMO-1", "Cross-B-N", "RUN-ALL-BENCH"}


def _read_json(path: Path, default: Any) -> Any:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (FileNotFoundError, json.JSONDecodeError, OSError):
        return default


def _sha256_json(value: Any) -> str:
    payload = json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True)
    return hashlib.sha256(payload.encode("utf-8")).hexdigest()


def canonical_id(plan_id: str) -> str:
    return ALIASES.get(plan_id, plan_id)


def discover_registry() -> dict[str, Any]:
    """Discover completion identities from plan filenames with the full corpus grammar."""
    found = {}
    base = ELAI_ROOT / "docs" / "thefinalglance"
    for plan_dir in sorted(base.glob("*-plan")):
        if not plan_dir.is_dir() or plan_dir.name == "patch-plan":
            continue
        for path in sorted(plan_dir.glob("*.md")):
            match = PLAN_ID_RE.match(path.name)
            if not match:
                continue
            raw_id = match.group(1)
            if INDEX_DOC_RE.match(path.name) or raw_id in SYNTHETIC_IDS:
                continue
            plan_id = canonical_id(raw_id)
            rel = path.relative_to(ELAI_ROOT).as_posix()
            if plan_id not in found:
                found[plan_id] = {"id": plan_id, "letter": plan_id[0], "path": rel}
            if raw_id != plan_id:
                found[plan_id].setdefault("aliases", []).append(raw_id)
    plans = [found[key] for key in sorted(found, key=_plan_sort_key)]
    registry = {
        "schema_version": REGISTRY_SCHEMA_VERSION,
        "identity_contract": "plan filenames; X-0 index documents and folded repair children excluded",
        "completion_units": len(plans),
        "aliases": ALIASES,
        "plans": plans,
    }
    registry["registry_revision"] = _sha256_json(registry)
    return registry


def _plan_sort_key(plan_id: str) -> tuple[int, int, int, int, str]:
    match = re.match(r"^([A-Q])-(\d+)(?:\.(\d+))?([a-z])?(.*)$", plan_id)
    if not match:
        return (99, 9999, 0, 0, plan_id)
    letter, number, fraction, suffix, tail = match.groups()
    return (
        ord(letter) - ord("A"),
        int(number),
        int(fraction or 0),
        ord(suffix or "`") - 96,
        tail,
    )


def write_registry(path: Path = REGISTRY_PATH) -> dict[str, Any]:
    registry = discover_registry()
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp = path.with_suffix(".json.tmp")
    tmp.write_text(json.dumps(registry, indent=2) + "\n", encoding="utf-8")
    tmp.replace(path)
    return registry


def load_registry(
    path: Path = REGISTRY_PATH, *, verify_corpus: bool = True
) -> dict[str, Any]:
    registry = _read_json(path, None)
    if not isinstance(registry, dict) or not isinstance(registry.get("plans"), list):
        raise ValueError(f"missing or malformed progress registry: {path}")
    expected = int(registry.get("completion_units", -1))
    if expected != len(registry["plans"]):
        raise ValueError("registry completion_units does not match plan rows")
    ids = [row.get("id") for row in registry["plans"]]
    if len(ids) != len(set(ids)):
        raise ValueError("registry contains duplicate plan identities")
    if verify_corpus:
        current = discover_registry()
        if current["registry_revision"] != registry.get("registry_revision"):
            missing = sorted(
                set(p["id"] for p in current["plans"]) - set(ids)
            )
            extra = sorted(
                set(ids) - set(p["id"] for p in current["plans"])
            )
            raise ValueError(
                f"plan registry drift; regenerate explicitly (missing={missing}, extra={extra})"
            )
    return registry


def _git_commit_is_landed(sha: str) -> bool:
    if not sha or not re.fullmatch(r"[0-9a-fA-F]{7,40}", sha):
        return False
    result = subprocess.run(
        [
            "git",
            "-C",
            str(ELAI_ROOT),
            "merge-base",
            "--is-ancestor",
            sha,
            "HEAD",
        ],
        capture_output=True,
        timeout=10,
    )
    return result.returncode == 0


def _durable_evidence_exists(ref: str) -> bool:
    """Accept only durable verifier/landing artifacts in the walk evidence stores."""
    if not ref or "/tmp" in ref or "\\tmp" in ref:
        return False
    candidates = re.findall(
        r"(?:/home/user/teikoku/[^\s,;)]+|conductor/[^\s,;)]+|opus_walk/[^\s,;)]+|\.elai_cc/[^\s,;)]+|workflow/[^\s,;)]+)",
        ref,
    )
    for raw in candidates:
        token = raw.rstrip(".:+]")
        if token.startswith("/home/user/teikoku/"):
            path = Path(token)
        elif token.startswith("conductor/"):
            path = TEIKOKU_ROOT / token
        elif token.startswith("opus_walk/"):
            path = TEIKOKU_ROOT / "conductor" / token
        else:
            path = ELAI_ROOT / token
        resolved = path.resolve()
        allowed = False
        for root in (WALK_ROOT / "evidence", WALK_ROOT / "dispatch-logs"):
            try:
                resolved.relative_to(root.resolve())
                allowed = True
            except ValueError:
                continue
            break
        if not allowed:
            continue
        artifact_re = re.compile(r"(?:gate|land|audit|verdict|result|proof)", re.I)
        if (
            resolved.is_file()
            and artifact_re.search(resolved.name)
            and resolved.stat().st_size > 0
        ):
            return True
        if not resolved.is_dir():
            continue
        for child in resolved.rglob("*"):
            if not child.is_file():
                continue
            if not artifact_re.search(child.name):
                continue
            if child.stat().st_size > 0:
                return True
    return False


def _load_plan_status() -> dict[str, dict[str, Any]]:
    out = {}
    if not PLAN_STATUS_DIR.is_dir():
        return out
    for path in PLAN_STATUS_DIR.glob("*.json"):
        row = _read_json(path, {})
        if not isinstance(row, dict):
            continue
        plan_id = canonical_id(str(row.get("plan_id") or path.stem))
        if plan_id in SYNTHETIC_IDS:
            continue
        out[plan_id] = row
    return out


def _load_card_states() -> dict[str, str]:
    """Rebuild analyst declarations in memory; never trust the walk-mutated plan-index."""
    if not CARD_INDEX_BUILDER.is_file():
        return {}
    code = "import json,sys;sys.path.insert(0,sys.argv[1]);import corpus_drain;print(json.dumps(corpus_drain.build_index()))"
    proc = subprocess.run(
        [sys.executable, "-c", code, str(WALK_ROOT)],
        capture_output=True,
        text=True,
        timeout=30,
        cwd=str(WALK_ROOT),
    )
    if proc.returncode != 0:
        return {}
    try:
        index = json.loads(proc.stdout)
    except json.JSONDecodeError:
        return {}
    return {
        canonical_id(str(row.get("id"))): str(row.get("state") or "uncertain")
        for row in index.get("units", [])
        if isinstance(row, dict) and row.get("id")
    }


def _load_audit_buckets() -> tuple[
    dict[str, str], dict[str, list[str]], dict[str, int]
]:
    data = _read_json(AUDIT_PATH, {})
    buckets = data.get("buckets", {}) if isinstance(data, dict) else {}
    mapping = {}
    normalized = {}
    raw_counts = {}
    for bucket, proof in (
        ("still_wired", "candidate"),
        ("stale_claim", "stale"),
        ("regressed", "rejected"),
        ("unparseable", "inconclusive"),
    ):
        raw_rows = buckets.get(bucket, []) if isinstance(buckets, dict) else []
        raw_counts[bucket] = len(raw_rows)
        ids = []
        for raw in raw_rows:
            plan_id = canonical_id(str(raw))
            if plan_id in SYNTHETIC_IDS:
                continue
            mapping[plan_id] = proof
            ids.append(plan_id)
        normalized[bucket] = sorted(set(ids), key=_plan_sort_key)
    return mapping, normalized, raw_counts


def _input_files() -> list[Path]:
    paths = [
        REGISTRY_PATH,
        DISPOSITIONS_PATH,
        WALK_STATE_PATH,
        AUDIT_PATH,
        CARD_INDEX_BUILDER,
    ]
    if PLAN_STATUS_DIR.is_dir():
        paths.extend(sorted(PLAN_STATUS_DIR.glob("*.json")))
    cards_dir = ELAI_ROOT / "workflow" / "reasoning" / "backlog-drain"
    if cards_dir.is_dir():
        paths.extend(sorted(cards_dir.glob("cards-*.md")))
    return [path for path in paths if path.exists()]


def _resolve_implementation_state(
    *,
    verified: bool,
    review_state: str,
    verdict: str,
    plan_status: str,
    declared: str,
) -> str:
    if verified:
        return "verified_delivered"
    if review_state == "escalated":
        return "blocked"
    if verdict in {"delivered", "drained", "claimed_delivered"} or plan_status == "DONE":
        return "claimed_delivered"
    if declared == "wired":
        return "declared_delivered"
    if declared == "partial":
        return "partial"
    if declared == "refuted":
        return "refuted"
    return "unstarted"


def build_status(registry_path: Path = REGISTRY_PATH) -> dict[str, Any]:
    registry = load_registry(registry_path)
    registry_ids = {row["id"] for row in registry["plans"]}
    dispositions_data = _read_json(DISPOSITIONS_PATH, {})
    raw_dispositions = (
        dispositions_data.get("dispositions", {})
        if isinstance(dispositions_data, dict)
        else {}
    )
    dispositions = {
        canonical_id(str(plan_id)): row
        for plan_id, row in raw_dispositions.items()
        if isinstance(row, dict)
        and canonical_id(str(plan_id)) in registry_ids
    }
    plan_status = _load_plan_status()
    card_states = _load_card_states()
    audit_proof, audit_buckets, audit_raw_counts = _load_audit_buckets()
    walk_state = _read_json(WALK_STATE_PATH, {})
    escalation_rows = (
        walk_state.get("escalation_queue", [])
        if isinstance(walk_state, dict)
        else []
    )
    escalations = {
        canonical_id(str(row.get("plan_id"))): row
        for row in escalation_rows
        if isinstance(row, dict)
        and canonical_id(str(row.get("plan_id"))) in registry_ids
    }
    rows = []
    punch = []
    for identity in registry["plans"]:
        plan_id = identity["id"]
        disposition = dispositions.get(plan_id, {})
        status = plan_status.get(plan_id, {})
        verdict = str(disposition.get("verdict") or "").lower()
        sha = str(status.get("commit_sha") or disposition.get("commit_sha") or "").strip()
        evidence = str(
            status.get("gate_evidence_ref") or disposition.get("evidence_ref") or ""
        ).strip()
        verified = (
            str(status.get("status") or "").upper() == "DONE"
            and _git_commit_is_landed(sha)
            and _durable_evidence_exists(evidence)
        )
        audit_state = audit_proof.get(plan_id)
        if verified:
            proof_state = "accepted"
        elif audit_state:
            proof_state = audit_state
        elif sha or evidence:
            proof_state = "candidate"
        else:
            proof_state = "missing"
        if verified:
            review_state = "settled"
        elif plan_id in escalations or verdict in {"escalated", "blocked", "set_aside"}:
            review_state = "escalated"
        elif disposition or status:
            review_state = "settled"
        else:
            review_state = "unreviewed"
        declared = card_states.get(plan_id, "uncertain")
        implementation_state = _resolve_implementation_state(
            verified=verified,
            review_state=review_state,
            verdict=verdict,
            plan_status=str(status.get("status") or "").upper(),
            declared=declared,
        )
        row = {
            "id": plan_id,
            "letter": identity["letter"],
            "implementation_state": implementation_state,
            "proof_state": proof_state,
            "review_state": review_state,
        }
        rows.append(row)
        if implementation_state == "blocked" or proof_state in {
            "rejected",
            "stale",
            "inconclusive",
        }:
            source = escalations.get(plan_id, {})
            reason = str(
                source.get("reason") or disposition.get("reason") or ""
            )
            if proof_state in {"rejected", "stale", "inconclusive"}:
                reason = f"proof {proof_state}: {reason}".strip()
            punch.append(
                {
                    "id": plan_id,
                    "desc": re.sub(r"\s+", " ", reason).strip()[:180]
                    or implementation_state,
                }
            )
    totals = Counter(row["implementation_state"] for row in rows)
    proof_totals = Counter(row["proof_state"] for row in rows)
    review_totals = Counter(row["review_state"] for row in rows)
    by_letter = defaultdict(Counter)
    for row in rows:
        by_letter[row["letter"]][row["implementation_state"]] += 1
        by_letter[row["letter"]][f"proof:{row['proof_state']}"] += 1
    tally = {}
    for letter, counts in sorted(by_letter.items()):
        plans = sum(
            counts[state]
            for state in (
                "verified_delivered",
                "claimed_delivered",
                "declared_delivered",
                "partial",
                "blocked",
                "refuted",
                "unstarted",
            )
        )
        tally[letter] = {
            "plans": plans,
            "verified": counts["verified_delivered"],
            "claimed_delivered": counts["claimed_delivered"],
            "declared_delivered": counts["declared_delivered"],
            "partial": counts["partial"],
            "blocked": counts["blocked"],
            "refuted": counts["refuted"],
            "unstarted": counts["unstarted"],
            "candidate_proof": counts["proof:candidate"],
            "wired": counts["verified_delivered"],
            "dark": counts["blocked"],
            "not_started": counts["unstarted"],
            "bench_gpu": 0,
            "other": counts["refuted"],
        }
    files = _input_files()
    source_mtime = int(max((path.stat().st_mtime for path in files), default=0))
    revision_input = {
        path.relative_to(TEIKOKU_ROOT).as_posix(): {
            "size": path.stat().st_size,
            "mtime_ns": path.stat().st_mtime_ns,
        }
        for path in files
    }
    return {
        "ok": True,
        "schema_version": STATUS_SCHEMA_VERSION,
        "source": "identity registry + live review ledger + landed proof + fresh analyst declarations",
        "source_mtime": source_mtime,
        "source_revision": _sha256_json(revision_input),
        "registry_revision": registry["registry_revision"],
        "total_units": len(rows),
        "tally": tally,
        "totals": dict(totals),
        "proof_totals": dict(proof_totals),
        "review_totals": dict(review_totals),
        "plan_rows": rows,
        "punch": sorted(punch, key=lambda item: _plan_sort_key(item["id"])),
        "open_count": len(punch),
        "audit_import": {
            "path": AUDIT_PATH.relative_to(TEIKOKU_ROOT).as_posix(),
            "buckets": audit_buckets,
            "raw_bucket_counts": audit_raw_counts,
            "canonical_bucket_counts": {
                key: len(value) for key, value in audit_buckets.items()
            },
            "contract": "still_wired imports as candidate, never accepted proof",
        },
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--write-registry", action="store_true")
    parser.add_argument("--status", action="store_true")
    args = parser.parse_args()
    if args.write_registry:
        registry = write_registry()
        print(
            f"wrote {REGISTRY_PATH} with {registry['completion_units']} completion units"
        )
    if args.status:
        print(json.dumps(build_status(), indent=2))
    if not args.write_registry and not args.status:
        parser.error("choose --write-registry or --status")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
