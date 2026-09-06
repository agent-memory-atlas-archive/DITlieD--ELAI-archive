#!/usr/bin/env python3
"""plan_selector.py - free-gated auto-selector that refills the opus walk from ELAI's open-gaps.

When the supervisor queue drains it HALTS and waits for an owner seed_plan (opus_supervisor
_no_actionable_work). This module is the inbound bridge that lets the walk refill ITSELF from
ELAI's own progress/wiring tracker instead of halting: it reads the open-gaps punch list
(elai_status_query.py --punch --json, the exact surface the status dashboard renders), applies a
FREE deterministic pre-filter, asks the FREE devin-bridge quorum which single gap is the smallest
EXECUTABLE seam fix, and returns that plan for walk_state.seed_plan. NO paid model is ever called
here (free-gated, owner choice 2026-07-12); the authoritative executable-vs-greenfield ruling stays
with the paid Opus S3 routing, which SET_ASIDEs anything greenfield -- this selector only avoids
feeding the obvious non-starters.

Safety posture -- fail CLOSED to a halt, never to a doubtful seed:
  * a gap already disposition=escalated|drained, already queued, already set-aside, or plan-DONE -> skip
  * a gap whose desc names a file currently dirty in /mnt/f -> skip: the land would dirty-overlap the
    owner's uncommitted WIP (the exact WALL-2 class that blocked CODEX-DEMO-1's first land)
  * a status snapshot older than AUTOSEED_MAX_SNAPSHOT_AGE_MIN -> select NOTHING (never drive a paid
    boot off stale progress data; the feeder/watcher must refresh elai-status.json first)
  * the free quorum naming NONE, or no majority -> select NOTHING (a halt is cheaper than a bad seed)

Stdlib only. Reuses quorum_triage's free roster + HTTP vote primitive (DRY). No paid turn here.
"""
from __future__ import annotations

import glob
import json
import os
import re
import subprocess
from pathlib import Path

import quorum_triage  # sibling: QUORUM_ROSTER + default_post (free bridge vote) + _plan_done
import walk_state      # sibling: the disposition ledger owner (dispositioned_ids)

# --- fixed wiring (named constants) ------------------------------------------
# Seed-attempt ledger (T6-entry10): a per-plan counter of how many times the walk has AUTO-SEEDED a
# candidate. The re-seed loop this kills: a plan the Opus confirms already-delivered but never
# dispositions keeps being re-selected, burning a full PAID turn each cycle (observed 2026-07-12:
# C-47 re-seeded 5 min after turn 2 confirmed it delivered). A candidate seeded >= RESEED_LIMIT times
# that STILL has no disposition row and is not plan-DONE is excluded from the pool (build_pool) until a
# disposition finally lands. Sidecar json beside walk-state.json, atomic-written like the state spine.
SEED_ATTEMPTS_PATH = Path(__file__).resolve().parent / "seed-attempts.json"
RESEED_LIMIT = 2
# The canonical ELAI workflow root on the teikoku distro (the /mnt/f share). Overridable for tests.
ELAI_WORKFLOW_ROOT = Path(os.environ.get("ELAI_WORKFLOW_ROOT", "/workspace/ELAI"))
# The Layer-0 status/wiring tracker that feeds the dashboard; --punch --json is its open-gaps read.
STATUS_QUERY = ELAI_WORKFLOW_ROOT / ".claude" / "tools" / "elai_status_query.py"
# The canonical git repo whose dirty set a candidate's land must not collide with (owner WIP guard).
ELAI_REPO = ELAI_WORKFLOW_ROOT
# Timeout for each git subprocess in the WIP guard. 60s (was 20s): `git status --porcelain` on the
# cold 9P /mnt/f mount is marginal at 20s, and a spurious timeout must not be mistaken for a git error.
GIT_GUARD_TIMEOUT_S = 60
# Do not select off a status snapshot older than this: stale progress data must never drive a paid
# boot. 180 min mirrors the elai rules' freshness window (elai_status_query --max-age-min default).
AUTOSEED_MAX_SNAPSHOT_AGE_MIN = 180.0
# Disposition verdicts that mean "not a fresh executable candidate": already blocked, or already done.
_SKIP_VERDICTS = ("escalated", "drained")
# Source-file token in a punch desc (basename match against the dirty set; conservative on collision).
_FILE_RE = re.compile(r"[\w./-]+\.(?:rs|svelte|ts|tsx|py|sh|toml|md)\b")
# A punch id we will actually seed must LOOK like a single plan unit (LETTER-.. -.., no whitespace):
# I-23, C-15, H-10, B-4, H-28-WIRE-1, CODEX-DEMO-1 qualify; free-text gaps like "H Track-3" or
# "glm-orchestrator dirt-baseline" do NOT (they are not a single executable plan the walk can run).
_PLAN_ID_RE = re.compile(r"^[A-Z][\w.]*-[\w.-]+$")
# The single blind question every voter answers when choosing the next executable gap (fail-closed).
# Hardened 2026-07-12 (orchestrator DECISION under handoff entry 10-fix): the live replay showed
# agentic voters (kimi-k2-7) answer with tool_use / "let me look around" instead of a ballot, so the
# question now explicitly FORBIDS tools/investigation and pins the answer to the FIRST LINE of the
# reply -- a machine-parseable ballot, not a conversation.
_SELECT_QUESTION = (
    "Which ONE of these open gaps is the SMALLEST, clearly EXECUTABLE fix that wires or activates an "
    "ALREADY-EXISTING seam (a dark symbol with a real existing production call site, an orphan "
    "component to mount, a regressed gate, a one-line plumbing/doc fix) -- NOT a greenfield build "
    "that needs a NEW seam designed first? Do NOT use tools, do NOT investigate or read anything; "
    "answer only from the listing above. The FIRST LINE of your reply MUST be exactly one plan id "
    "from the list, or the word NONE if none is a safe small executable fix -- nothing before it."
)
# Ballot cap: the free quorum votes over only the TOP-K of the (already priority-ordered) pool, so
# the F-1 expansion to ~40 near-equal candidates can no longer split the ~2 reliable voters into a
# structural tie. The pool order IS the deterministic preference (punch head, then audited-open,
# then recon-first), so the top-K is the strongest slice, and a smaller ballot keeps the blind-vote
# prompt short + the votes concentrated.
SELECT_BALLOT_MAX = 5
# Ballot ROTATION (2026-07-17 head-of-line fix): the ballot cap alone deadlocks when the top-K of a
# static drain order is stable-rejected -- observed live: 4 consecutive ALL_NONE on the SAME 5 heads
# (D-28/E-7/E-14/E-15/E-20) while ~300 voteable units sat below the window, walking the chain toward
# the 12-strike park with the backlog full. Fix: an ALL_NONE verdict advances a persisted window
# offset by the ballot size, so the NEXT cycle votes on the next K units (wrapping). Punch-tier units
# (the live curated head) are never rotated out -- they stay on every ballot; only the corpus/summary
# tail rotates. PICK/TIE do not move the window (the winner leaves the pool on its own); NO_PICK and
# QUORUM_THIN do not move it either (those are voter-infra failures, not a judgment on the units).
# Sidecar json beside walk-state.json, same atomic-write shape as seed-attempts.
ROTATION_PATH = Path(__file__).resolve().parent / "quorum-rotation.json"


def _rotation_offset(path: Path | None = None) -> int:
    """Current ballot window offset into the non-punch pool tail (0 when absent/unreadable).
    path resolves against the module ROTATION_PATH at call time so tests can point it at a tmp file."""
    path = path or ROTATION_PATH
    try:
        val = json.loads(path.read_text(encoding="utf-8")).get("offset", 0)
        return val if isinstance(val, int) and val >= 0 else 0
    except (OSError, ValueError):
        return 0


def _advance_rotation(step: int, tail_len: int, path: Path | None = None) -> int:
    """Advance the window by `step`, wrapping over the current tail length. Returns the new offset.
    Best-effort persistence: a write failure just means the next cycle re-votes the same window."""
    path = path or ROTATION_PATH
    new = (_rotation_offset(path) + max(step, 1)) % tail_len if tail_len else 0
    try:
        path.write_text(json.dumps({"offset": new}), encoding="utf-8")
    except OSError:
        pass
    return new
# Thin-quorum floor: at least this many PARSEABLE votes (a valid id pick OR an explicit NONE
# abstention -- both prove the voter engaged with the ballot) are required before any verdict beyond
# a halt is trusted. Fewer means the voters are actually unreachable/unparseable/derailed, not merely
# split -> halt fail-closed (a bad seed costs a PAID Opus boot). Plurality + tie-break then run over
# id-picks only; a floor-met quorum that is ALL NONE still halts (ALL_NONE) -- genuine "nothing
# actionable" stays fail-closed. Distinct from quorum_triage.QUORUM_MIN_VOTES (that wake gate is
# fail-OPEN; this seed gate is fail-CLOSED).
SELECT_QUORUM_MIN = 2

# --- F-1 pool expansion: SUMMARY-index enumeration + synthesis wire-dark tier ------------
# The full plan corpus lives here, one dir per letter, each with a per-unit SUMMARY index. When the
# curated --punch head drains, the walk refills from these per-unit rows instead of halting.
SUMMARY_DIR = ELAI_WORKFLOW_ROOT / "docs" / "thefinalglance"
# Letters we enumerate. D's SUMMARY index is wave-grain (### Wave N, not ### D-N), so it yields zero
# parseable units and is reported as an excluded letter -- expected, not an error.
PLAN_LETTERS = tuple("BCDEFGHIJKLMNOPQ")
# The newest plan-audit synthesis. Its per-unit rows are prose, not a clean table (the P0 punch list
# it carries IS the --punch head), so per the declared FORK we use it ONLY as a NAMED-IDS signal: a
# unit id explicitly named in the synthesis = audit-confirmed OPEN/defect -> a higher pool tier than
# pure enumeration. A letter with no named ids falls back to SUMMARY enumeration alone (recon-first).
SYNTHESIS_ROOT = ELAI_WORKFLOW_ROOT / "workflow" / "reasoning"
SYNTHESIS_GLOB = "plan-audit-*/_MASTER-SYNTHESIS.md"
# Cap on summary-derived candidates (the --punch head is always kept ON TOP of this, uncapped) so the
# free-quorum prompt stays tractable: audited-open units are kept first, the recon-first tail fills up
# to the cap. ~40 short rows keeps the blind-vote prompt around ~50 lines.
SUMMARY_POOL_MAX = 40
# When conductor/CODEX_FIRST is present, prefer these letters in the summary pool
# so paid codex credits hit J+ before months of already-shipped C recon.
CODEX_BURN_LETTERS = tuple("JKLMNOPQ")
CODEX_FIRST_SENTINEL = Path(__file__).resolve().parents[1] / "CODEX_FIRST"
# The first whitespace token after '### ' is the unit id ('E-1', 'B-1:', ...); a trailing [..] bracket
# in a header carries B's inline status verdict (### B-1: Title  [DONE s-59964]).
_HDR_ID_RE = re.compile(r"^###\s+(\S+)")
_HDR_BRACKET_RE = re.compile(r"\[([^\]]*)\]\s*$")
# Status verdicts that mean "not fresh activation fodder" (drop) vs eligible. PARTIAL wins over a
# co-mentioned DONE ('[PARTIAL -- main.rs decomposition DONE]' is PARTIAL, eligible).
_SUMMARY_DROP_STATUS = ("DONE", "DEFERRED")


def _atomic_write_json(path: str, data: dict) -> None:
    """Write data to path atomically (tmp in the same dir + os.replace). Mirrors walk_state's spine."""
    p = Path(path)
    p.parent.mkdir(parents=True, exist_ok=True)
    tmp = p.with_name(f"{p.name}.{os.getpid()}.tmp")
    try:
        tmp.write_text(json.dumps(data, indent=2, sort_keys=True), encoding="utf-8")
        os.replace(tmp, p)
    except BaseException:
        try:
            tmp.unlink()
        except OSError:
            pass
        raise


def load_seed_attempts(path: str = str(SEED_ATTEMPTS_PATH)) -> dict:
    """The per-plan auto-seed counts, {plan_id: int}. Fail-safe: a missing or unreadable ledger is an
    empty dict (never raises), so the reseed filter simply excludes nothing when the ledger is absent."""
    try:
        data = json.loads(Path(path).read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return {}
    if not isinstance(data, dict):
        return {}
    return {k: int(v) for k, v in data.items() if isinstance(v, (int, float))}


def record_seed_attempt(plan_id: str, path: str = str(SEED_ATTEMPTS_PATH)) -> int:
    """Increment and persist the auto-seed count for plan_id; return the new count. Called by the
    supervisor's _try_autoseed on every successful seed so the reseed filter can bound re-selection."""
    counts = load_seed_attempts(path)
    counts[plan_id] = int(counts.get(plan_id, 0)) + 1
    _atomic_write_json(path, counts)
    return counts[plan_id]


def _drop_reseeded(pool: list, plan_status_dir: str, seed_attempts: dict, dispositioned: set) -> tuple:
    """Exclude any candidate seeded >= RESEED_LIMIT times that STILL has no disposition row AND is not
    plan-DONE -- the structural kill for the re-seed loop (a confirmed-already-delivered plan that was
    never dispositioned kept getting re-selected, one paid turn burned per cycle). A plan that HAS a
    disposition (drained/escalated) is governed by that disposition, not the reseed cap, so it is kept
    here (and already dropped upstream by eligible_gaps when its punch verdict is terminal). Logs one
    SKIP_RESEED line per exclusion. Returns (kept_pool, skipped_ids)."""
    kept, skipped = [], []
    for g in pool:
        pid = g["id"]
        n = int(seed_attempts.get(pid, 0))
        if (n >= RESEED_LIMIT and pid not in dispositioned
                and not quorum_triage._plan_done(pid, plan_status_dir)):
            print(f"[plan_selector] SKIP_RESEED {pid} (seeded {n}x, no disposition)", flush=True)
            skipped.append(pid)
            continue
        kept.append(g)
    return kept, skipped


def _classify_summary_status(bracket: str) -> str:
    """Map a header's trailing [..] bracket to DONE|PARTIAL|PLANNED|DEFERRED|OPEN. OPEN = no status
    verdict in the header (the common non-B case; those letters carry no inline status). PARTIAL is
    checked before DONE so a '[PARTIAL ... DONE]' bracket classifies PARTIAL, not DONE."""
    u = (bracket or "").upper()
    if "DEFERRED" in u:
        return "DEFERRED"
    if "PARTIAL" in u:
        return "PARTIAL"
    if "DONE" in u:
        return "DONE"
    if "PLANNED" in u:
        return "PLANNED"
    return "OPEN"


def _parse_summary_file(path: Path) -> tuple:
    """Parse ONE SUMMARY index into per-unit candidates. Returns (units, read_ok). Defensive: a header
    whose first token is not a real plan-id (### Wave 1, free text) is skipped; a whole-file read error
    yields ([], False) so the caller excludes+logs that letter. Each unit -> {id, desc, files, status}.
    files is every source-file basename referenced anywhere in the unit's block (reused _FILE_RE), so
    the dirty-WIP guard still applies regardless of the letter's row layout."""
    try:
        text = path.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return [], False
    lines = text.splitlines()
    starts = [i for i, ln in enumerate(lines) if ln.startswith("### ")]
    units = []
    for n, start in enumerate(starts):
        end = starts[n + 1] if n + 1 < len(starts) else len(lines)
        header = lines[start]
        m = _HDR_ID_RE.match(header)
        if not m:
            continue
        pid = m.group(1).rstrip(":.,").strip()
        if not _PLAN_ID_RE.match(pid):
            continue  # 'Wave', free-text, or a non-unit header -> not a seedable plan unit
        bracket = _HDR_BRACKET_RE.search(header)
        status = _classify_summary_status(bracket.group(1) if bracket else "")
        title = _HDR_BRACKET_RE.sub("", header[m.end():]).strip(" —-:").strip()
        files = _extract_files("\n".join(lines[start:end]))
        units.append({"id": pid, "desc": (title or pid)[:200],
                      "files": sorted(files), "status": status})
    return units, True


def summary_candidates(summary_dir: Path = SUMMARY_DIR) -> tuple:
    """Enumerate per-unit candidates across every letter's SUMMARY index. Returns (units, excluded):
    units is the flat list of {id,desc,files,status,letter}; excluded is the letters that produced zero
    parseable units (missing file, wave-grain index, or read error) for the pool log. Never raises -- a
    bad letter is excluded, not fatal (the STATUS side, not this, holds the fail-CLOSED halt posture)."""
    units, excluded = [], []
    for letter in PLAN_LETTERS:
        hits = sorted(glob.glob(str(Path(summary_dir) / f"{letter}-plan" / "*SUMMARY*.md")))
        got = 0
        for h in hits:
            parsed, read_ok = _parse_summary_file(Path(h))
            if not read_ok:
                continue
            for u in parsed:
                u["letter"] = letter
                units.append(u)
                got += 1
        if got == 0:
            excluded.append(letter)
    return units, excluded


def _newest_synthesis(root: Path = SYNTHESIS_ROOT) -> Path | None:
    """The newest plan-audit _MASTER-SYNTHESIS.md, or None if none exists."""
    hits = glob.glob(str(Path(root) / SYNTHESIS_GLOB))
    if not hits:
        return None
    return Path(max(hits, key=os.path.getmtime))


def synthesis_named_ids(synthesis_path: Path | None = None) -> set:
    """The set of plan-ids EXPLICITLY NAMED in the newest audit synthesis (its punch list + the itemized
    PARTIAL/DARK/NOT_STARTED prose). Membership = audit-confirmed OPEN/defect -> the audited-open pool
    tier (higher than pure enumeration). Best-effort per the declared FORK: no synthesis or a read error
    yields an empty set (every summary unit then falls to the recon-first tier), never an exception."""
    p = synthesis_path if synthesis_path is not None else _newest_synthesis()
    if not p:
        return set()
    try:
        text = Path(p).read_text(encoding="utf-8", errors="replace")
    except OSError:
        return set()
    return {tok for tok in re.findall(r"\b[A-Z]-\d+(?:-[A-Za-z0-9]+)*\b", text)
            if _PLAN_ID_RE.match(tok)}


def load_corpus_order() -> tuple:
    """Production corpus reader: (ordered_build_ids, plan_index) from corpus_drain's full-corpus,
    supersession-aware ordered drain. Guarded end-to-end -- a missing corpus_drain module, an absent
    plan-index.json, or any parse failure yields ([], None), so the selector falls back to the
    punch+SUMMARY behavior and NEVER crashes (the walk must run with no cards). This is the default
    corpus_reader the production caller (_try_autoseed) injects into select_next; tests leave it off."""
    try:
        import corpus_drain  # sibling; lazy so a broken cards dir never breaks import of this module
        index = corpus_drain.load_index()
        ordered, _actions = corpus_drain.ordered_drain(index)
        return ordered, index
    except Exception:  # noqa: BLE001 - corpus tier is additive; any failure falls back to punch+SUMMARY
        return [], None


def _corpus_gaps(corpus_ordered: list, corpus_index: dict | None, seen: set, dispositioned: set,
                 queued: set, escalated: set, dirty: set, plan_status_dir: str) -> list:
    """The corpus tier: the full-corpus supersession-ordered drain (uncapped), each id passed through
    the SAME eligibility the punch/summary tiers use (already-seen, dispositioned, queued, escalated,
    plan-DONE, dirty-WIP overlap). Returns a list of {id, desc, files, recon_first} in drain order and
    adds every kept id to `seen` so the SUMMARY tier only contributes UNCARDED units. An empty order
    or a None index yields [] (the pool is then exactly the legacy punch+SUMMARY)."""
    if not corpus_ordered or not corpus_index:
        return []
    by_id = {u.get("id"): u for u in corpus_index.get("units", [])}
    gaps = []
    for pid in corpus_ordered:
        if pid in seen or pid in dispositioned or pid in queued or pid in escalated:
            continue
        if quorum_triage._plan_done(pid, plan_status_dir):
            continue
        u = by_id.get(pid) or {}
        files = sorted(_extract_files(" ".join(u.get("key_files") or []) + " " + (u.get("does") or "")))
        if set(files) & dirty:
            continue  # would dirty-overlap owner WIP (same WALL-2 guard as the other tiers)
        # a corpus unit graded 'uncertain' still needs its settling recon before a write (RECON class).
        recon_first = (u.get("state") == "uncertain")
        gaps.append({"id": pid, "desc": (u.get("does") or pid)[:200], "files": files,
                     "recon_first": recon_first})
        seen.add(pid)
    return gaps


AUDIT_INDEX_PATH = Path(__file__).resolve().parent / "evidence" / "batch-recon-2026-08-index.json"
AUDIT_ALLOWED_VERDICTS = frozenset(("EXECUTE-AS-IS",))
SETTLED_TERMINAL_VERDICTS = frozenset((
    "drained", "verified_delivered", "claimed_delivered", "superseded", "already_done",
))


def load_audited_index(index_path: Path | str | None = None,
                       plan_status_dir: str | Path | None = None) -> list:
    """Load strict executable audit rows and pin their evidence and citation sources."""
    import hashlib
    from datetime import datetime

    index_path = Path(index_path) if index_path is not None else AUDIT_INDEX_PATH
    status_dir = (Path(plan_status_dir) if plan_status_dir is not None
                  else Path(__file__).resolve().parent / "plan-status")
    report = {
        "index_path": str(index_path),
        "rows": 0,
        "admitted": 0,
        "settled": 0,
        "excluded": [],
        "exclusions": {
            "AMEND": 0,
            "OWNER-GATED": 0,
            "ALREADY-DONE": 0,
            "SUPERSEDED-BY": 0,
            "OTHER": 0,
            "SCHEMA": 0,
            "MISSING-EVIDENCE": 0,
            "audit_evidence_drift": 0,
            "citation_source_drift": 0,
        },
        "all_settled": {"complete": False, "valid_execute_rows": 0, "unsettled": []},
    }
    load_audited_index.last_report = report

    def exclude(plan_id, class_name, reason):
        report["exclusions"][class_name] = report["exclusions"].get(class_name, 0) + 1
        report["excluded"].append({"id": plan_id, "class": class_name, "reason": reason})

    def digest(raw):
        return hashlib.sha256(raw).hexdigest()

    def normalized_line(raw, line_number):
        lines = raw.splitlines(keepends=True)
        if line_number <= 0 or line_number > len(lines):
            raise ValueError("citation_line_out_of_range")
        value = lines[line_number - 1]
        if value.endswith(b"\r\n"):
            value = value[:-2]
        elif value.endswith(b"\n") or value.endswith(b"\r"):
            value = value[:-1]
        return value.decode("utf-8")

    def reference_exists(value):
        if not isinstance(value, str) or not value.strip():
            return False
        root = Path(__file__).resolve().parents[2]
        for part in value.split(" + "):
            text = part.strip()
            if not text:
                continue
            candidates = [Path(text)] if Path(text).is_absolute() else [
                root / text, index_path.parent / text, Path.cwd() / text,
            ]
            if any(candidate.is_file() for candidate in candidates):
                return True
        return False

    try:
        data = json.loads(index_path.read_bytes())
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as exc:
        report["index_error"] = f"{type(exc).__name__}:{exc}"
        load_audited_index.last_report = report
        return []
    rows = data.get("rows") if isinstance(data, dict) else None
    if not isinstance(rows, list):
        report["index_error"] = "rows_missing_or_not_list"
        load_audited_index.last_report = report
        return []
    report["rows"] = len(rows)
    valid_execute = []
    source_cache = {}

    for row in rows:
        if not isinstance(row, dict):
            exclude("(invalid-row)", "SCHEMA", "index_row_not_object")
            continue
        raw_id = row.get("id")
        plan_id = raw_id if isinstance(raw_id, str) and raw_id.strip() else "(missing-id)"
        verdict = row.get("verdict")
        if verdict not in AUDIT_ALLOWED_VERDICTS:
            if isinstance(verdict, str) and verdict.startswith("SUPERSEDED-BY"):
                class_name = "SUPERSEDED-BY"
            elif verdict in ("AMEND", "OWNER-GATED", "ALREADY-DONE"):
                class_name = verdict
            else:
                class_name = "OTHER"
            exclude(plan_id, class_name, "index_verdict_not_admissible")
            continue

        evidence_path = row.get("evidence_path")
        if not isinstance(evidence_path, str) or not evidence_path.strip():
            exclude(plan_id, "MISSING-EVIDENCE", "evidence_path_missing")
            continue
        evidence_file = Path(evidence_path)
        try:
            evidence_raw = evidence_file.read_bytes()
            evidence = json.loads(evidence_raw)
        except (OSError, UnicodeDecodeError, json.JSONDecodeError) as exc:
            exclude(plan_id, "MISSING-EVIDENCE", f"evidence_unreadable:{type(exc).__name__}")
            continue

        reason = None
        if not isinstance(evidence, dict):
            reason = "evidence_not_object"
        elif evidence.get("id") != plan_id:
            reason = "evidence_id_mismatch"
        elif evidence.get("verdict") != "EXECUTE-AS-IS":
            reason = "evidence_verdict_mismatch"
        elif not isinstance(evidence.get("citations"), list) or not evidence["citations"]:
            reason = "citations_empty"
        elif not isinstance(evidence.get("rationale"), str) or not evidence["rationale"].strip():
            reason = "rationale_empty"
        else:
            try:
                datetime.fromisoformat(str(evidence.get("recon_ts")).replace("Z", "+00:00"))
            except (TypeError, ValueError):
                reason = "recon_ts_unparseable"

        pins = []
        bound = False
        if reason is None:
            for citation in evidence["citations"]:
                if not isinstance(citation, dict):
                    reason = "citation_not_object"
                    break
                source = citation.get("file")
                line_number = citation.get("line")
                stored_text = citation.get("text")
                if (not isinstance(source, str) or not source.strip()
                        or not isinstance(line_number, int) or isinstance(line_number, bool)
                        or line_number <= 0 or not isinstance(stored_text, str)
                        or not isinstance(citation.get("why"), str)
                        or not citation.get("why", "").strip()):
                    reason = "citation_shape_invalid"
                    break
                try:
                    source_raw, source_sha = source_cache.get(source, (None, None))
                    if source_raw is None:
                        source_raw = Path(source).read_bytes()
                        source_sha = digest(source_raw)
                        source_cache[source] = (source_raw, source_sha)
                    actual_text = normalized_line(source_raw, line_number)
                except (OSError, UnicodeDecodeError, ValueError):
                    reason = "citation_source_invalid"
                    break
                if actual_text != stored_text:
                    reason = "citation_text_not_byte_exact"
                    break
                if re.search(rf"(?<![\w-]){re.escape(plan_id)}(?![\w-])", actual_text):
                    bound = True
                pins.append({"file": source, "sha256": source_sha,
                             "line": line_number, "text": actual_text})

        if reason is not None:
            exclude(plan_id, "SCHEMA", reason)
            continue
        if not bound:
            exclude(plan_id, "SCHEMA", "citation_plan_id_binding_missing")
            continue
        valid_execute.append({
            "id": plan_id,
            "desc": evidence["rationale"].strip()[:200],
            "files": sorted({os.path.basename(pin["file"]) for pin in pins}),
            "recon_first": False,
            "tier": "audited",
            "verdict": "EXECUTE-AS-IS",
            "audit_evidence_path": str(evidence_file),
            "audit_evidence_sha256": digest(evidence_raw),
            "audit_citations": pins,
            "_audit_report": report,
        })

    dispositions = {}
    ledger_path = status_dir.parent / "plan-dispositions.json"
    try:
        ledger = json.loads(ledger_path.read_text(encoding="utf-8"))
        if isinstance(ledger, dict) and isinstance(ledger.get("dispositions"), dict):
            dispositions = ledger["dispositions"]
    except (OSError, UnicodeDecodeError, json.JSONDecodeError):
        dispositions = {}

    def settled_proof(entry):
        plan_id = entry["id"]
        disposition = dispositions.get(plan_id)
        if not isinstance(disposition, dict):
            return "disposition_missing"
        if disposition.get("plan_id") != plan_id:
            return "disposition_plan_id_mismatch"
        terminal = disposition.get("verdict")
        if terminal not in SETTLED_TERMINAL_VERDICTS:
            return "disposition_verdict_not_frozen"
        if not reference_exists(disposition.get("evidence_ref")):
            return "disposition_evidence_ref_missing"
        if terminal == "superseded":
            target = disposition.get("superseding_id") or disposition.get("superseded_by")
            if not target and isinstance(disposition.get("supersedence"), dict):
                target = (disposition["supersedence"].get("superseding_id")
                          or disposition["supersedence"].get("target_id"))
            return None if isinstance(target, str) and target.strip() else "supersedence_target_proof_missing"
        status_path = status_dir / f"{plan_id}.json"
        try:
            status = json.loads(status_path.read_text(encoding="utf-8"))
        except (OSError, UnicodeDecodeError, json.JSONDecodeError):
            return "plan_status_missing"
        if (not isinstance(status, dict) or str(status.get("status", "")).upper() != "DONE"
                or not isinstance(status.get("commit_sha"), str)
                or not status.get("commit_sha", "").strip()):
            return "done_commit_proof_missing"
        return None

    unsettled = []
    open_admitted = []
    for entry in valid_execute:
        reason = settled_proof(entry)
        if reason:
            unsettled.append({"id": entry["id"], "reason": reason})
            open_admitted.append(entry)
        else:
            report["settled"] += 1
    report["admitted"] = len(open_admitted)
    report["all_settled"] = {
        "complete": bool(valid_execute) and not unsettled,
        "valid_execute_rows": len(valid_execute),
        "unsettled": unsettled,
    }
    load_audited_index.last_report = report
    return open_admitted

def build_pool(status: dict, state: dict, dirty: set, plan_status_dir: str,
               summary_dir: Path = SUMMARY_DIR, synthesis_path: Path | None = None,
               seed_attempts: dict | None = None, dispositioned: set | None = None,
               punch_stale: bool = False, corpus_ordered: list | None = None,
               corpus_index: dict | None = None) -> tuple:
    """Build the ordered punch, audited, corpus, and summary candidate pool."""
    if dispositioned is None:
        dispositioned = walk_state.dispositioned_ids()

    if punch_stale:
        punch, punch_disp_drop = [], 0
    else:
        punch_raw = eligible_gaps(status, state, dirty, plan_status_dir)
        punch = [g for g in punch_raw if g["id"] not in dispositioned]
        punch_disp_drop = len(punch_raw) - len(punch)
    for gap in punch:
        gap["recon_first"] = False

    queued = set(state.get("queue") or [])
    escalated = {entry.get("plan_id") for entry in (state.get("escalation_queue") or [])
                 if entry.get("plan_id")}
    seen = {gap["id"] for gap in punch}

    audit_enabled = Path(plan_status_dir).is_dir()
    if audit_enabled:
        audited_entries = load_audited_index(plan_status_dir=plan_status_dir)
        audit_report = getattr(load_audited_index, "last_report", {})
    else:
        audited_entries = []
        audit_report = {
            "index_path": "",
            "exclusions": {},
            "all_settled": {"complete": False, "valid_execute_rows": 0, "unsettled": []},
        }
    audited_pool = []
    audit_skipped = 0
    for gap in audited_entries:
        pid = gap["id"]
        if pid in seen:
            continue
        if pid in dispositioned:
            audit_skipped += 1
            continue
        if pid in queued or pid in escalated:
            continue
        if quorum_triage._plan_done(pid, plan_status_dir):
            continue
        if set(gap["files"]) & dirty:
            continue
        seen.add(pid)
        audited_pool.append(gap)

    corpus_gaps = _corpus_gaps(corpus_ordered or [], corpus_index, seen, dispositioned,
                               queued, escalated, dirty, plan_status_dir)
    named = synthesis_named_ids(synthesis_path)
    units, excluded = summary_candidates(summary_dir)
    summary_audited, recon = [], []
    disposition_skipped = 0
    for unit in units:
        pid = unit["id"]
        if pid in seen:
            continue
        if pid in dispositioned:
            disposition_skipped += 1
            continue
        seen.add(pid)
        if unit["status"] in _SUMMARY_DROP_STATUS:
            continue
        if pid in queued or pid in escalated:
            continue
        if quorum_triage._plan_done(pid, plan_status_dir):
            continue
        if set(unit["files"]) & dirty:
            continue
        is_audited = pid in named or unit["status"] == "PARTIAL"
        gap = {"id": pid, "desc": unit["desc"], "files": unit["files"],
               "recon_first": not is_audited}
        (summary_audited if is_audited else recon).append(gap)

    def _letter(pid: str) -> str:
        return (pid or "").split("-", 1)[0]

    if CODEX_FIRST_SENTINEL.is_file():
        burn_a = [gap for gap in summary_audited if _letter(gap["id"]) in CODEX_BURN_LETTERS]
        rest_a = [gap for gap in summary_audited if _letter(gap["id"]) not in CODEX_BURN_LETTERS]
        burn_r = [gap for gap in recon if _letter(gap["id"]) in CODEX_BURN_LETTERS]
        rest_r = [gap for gap in recon if _letter(gap["id"]) not in CODEX_BURN_LETTERS]
        ordered = burn_a + burn_r + rest_a + rest_r
        summary_pool = ordered[:SUMMARY_POOL_MAX]
    else:
        summary_pool = (summary_audited + recon)[:SUMMARY_POOL_MAX]

    pool = punch + audited_pool + corpus_gaps + summary_pool
    stats = {
        "punch": len(punch),
        "audited": len(audited_pool),
        "corpus": len(corpus_gaps),
        "summary": len(summary_pool),
        "audited_open": (len(audited_pool) if audit_enabled else len(summary_audited)),
        "recon_first": sum(1 for gap in summary_pool if gap["recon_first"]),
        "summary_total_eligible": len(summary_audited) + len(recon),
        "excluded_letters": excluded,
        "audit_exclusions": audit_report.get("exclusions", {}),
        "audit_all_settled": audit_report.get("all_settled", {
            "complete": False, "valid_execute_rows": 0, "unsettled": [],
        }),
        "dispositioned": disposition_skipped + audit_skipped + punch_disp_drop,
    }
    if seed_attempts is None:
        seed_attempts = load_seed_attempts()
    pool, skipped_reseed = _drop_reseeded(pool, plan_status_dir, seed_attempts, dispositioned)
    stats["skipped_reseed"] = skipped_reseed
    return pool, stats


def read_status(query_path: Path = STATUS_QUERY) -> dict | None:
    """Run elai_status_query --punch --json and return the parsed dict, or None on any failure
    (missing tool, non-zero exit, unparseable JSON). Honest-degrade at the boundary: a status
    tracker we cannot read means we do NOT autoseed (the caller halts), never a guessed seed."""
    if not query_path.is_file():
        return None
    try:
        out = subprocess.run(
            ["python3", str(query_path), "--punch", "--json"],
            capture_output=True, text=True, timeout=30, check=False,
        )
    except (OSError, subprocess.SubprocessError):
        return None
    if out.returncode not in (0, 3):  # 0 fresh, 3 stale-but-emitted; both carry a punch payload
        return None
    try:
        data = json.loads(out.stdout)
    except (json.JSONDecodeError, ValueError):
        return None
    return data if isinstance(data, dict) else None


def dirty_basenames(repo: Path = ELAI_REPO) -> set | None:
    """The basenames of every path git reports dirty (tracked, untracked, OR parked in the top stash)
    in repo. A candidate gap that names any of these files is skipped: landing a change to it would
    dirty-overlap the owner's uncommitted WIP, and stashed WIP must not collide with a later pop
    (WALL-2). Basename match is deliberately conservative -- a same-name collision skips a safe plan,
    which is the safe direction.

    The working-tree leg and the stash leg run INDEPENDENTLY: a failure of one NEVER drops the other,
    so a transient `git status` timeout can no longer silently disable the stash guard (the defect this
    signature exists to prevent). Fail-CLOSED floor: if BOTH legs fail (the guard is fully blind), this
    returns None so the caller HALTS rather than seed off an unverifiable WIP set. A None from either
    single leg is treated as 'that leg contributed nothing'; only both-None is a blind guard."""
    status_names = _git_status_basenames(repo)
    stash_names = _stashed_basenames(repo)
    if status_names is None and stash_names is None:
        return None  # guard fully blind: both legs errored -> caller must halt (fail-closed)
    names: set = set()
    if status_names is not None:
        names |= status_names
    if stash_names is not None:  # parked WIP is protected too (a later pop must not collide)
        names |= stash_names
    return names


def _git_status_basenames(repo: Path = ELAI_REPO) -> set | None:
    """Basenames of every path `git status --porcelain` reports dirty (tracked or untracked). Returns
    the set on success, or None if the leg FAILED (subprocess error/timeout or nonzero rc) so the
    caller can tell 'clean tree' (empty set) from 'could not read the tree' (None). Independent of the
    stash leg: its failure must not drop the stash guard."""
    try:
        out = subprocess.run(
            ["git", "-C", str(repo), "status", "--porcelain"],
            capture_output=True, text=True, timeout=GIT_GUARD_TIMEOUT_S, check=False,
        )
    except (OSError, subprocess.SubprocessError):
        return None
    if out.returncode != 0:
        return None
    names = set()
    for line in out.stdout.splitlines():
        if not line.strip():
            continue
        path = line[3:] if len(line) > 3 else line
        if " -> " in path:  # rename: XY old -> new
            path = path.split(" -> ", 1)[1]
        names.add(os.path.basename(path.strip().strip('"')))
    return names


def _stashed_basenames(repo: Path = ELAI_REPO) -> set | None:
    """Basenames of files touched by the TOP git stash (the most-recently-parked owner WIP). Included
    in the dirty guard so a gap that would land into a file the owner has stashed is skipped -- the
    stash pop would otherwise conflict on it. Returns None ONLY when the subprocess itself failed
    (error/timeout) -- a genuine leg failure. A clean nonzero rc (no stash / bad ref) is NOT a failure:
    it means 'nothing parked' and returns an empty set, so 'no stash' never counts toward the
    both-legs-blind halt."""
    try:
        out = subprocess.run(
            ["git", "-C", str(repo), "stash", "show", "--name-only", "stash@{0}"],
            capture_output=True, text=True, timeout=GIT_GUARD_TIMEOUT_S, check=False,
        )
    except (OSError, subprocess.SubprocessError):
        return None  # real leg failure (timeout / git unavailable) -> contributes nothing, may halt
    if out.returncode != 0:  # no stash / bad ref -> nothing parked (NOT a failure)
        return set()
    return {os.path.basename(p.strip()) for p in out.stdout.splitlines() if p.strip()}


def _extract_files(desc: str) -> set:
    """The set of source-file basenames a punch desc references (e.g. 'production_agent_rpc.rs')."""
    return {os.path.basename(m.group(0)) for m in _FILE_RE.finditer(desc or "")}


def eligible_gaps(status: dict, state: dict, dirty: set, plan_status_dir: str) -> list:
    """The deterministically-eligible open gaps: not escalated/drained/queued/set-aside/plan-DONE,
    and not touching any dirty (WIP) file. Each -> {id, desc, files}. This is the FREE pre-filter;
    the free quorum then picks one of these and the paid Opus S3 route makes the final call."""
    queued = set(state.get("queue") or [])
    escalated = {e.get("plan_id") for e in (state.get("escalation_queue") or []) if e.get("plan_id")}
    out = []
    for gap in status.get("punch") or []:
        pid = gap.get("id")
        if not pid or not isinstance(pid, str):
            continue
        if not _PLAN_ID_RE.match(pid):
            continue  # free-text/track-level gap, not a single seedable plan unit
        verdict = (gap.get("disposition") or {}).get("verdict")
        if verdict in _SKIP_VERDICTS:
            continue
        if pid in queued or pid in escalated:
            continue
        if quorum_triage._plan_done(pid, plan_status_dir):
            continue
        files = _extract_files(gap.get("desc") or "")
        if files & dirty:
            continue
        out.append({"id": pid, "desc": (gap.get("desc") or "").strip(), "files": sorted(files)})
    return out


def _select_prompt(eligible: list) -> str:
    """The identical blind prompt handed to every free voter to pick the next executable gap."""
    listing = "\n".join(f"- {g['id']}: {g['desc']}" for g in eligible)
    return ("You are one blind voter helping a backlog walk pick its NEXT plan. You cannot see the "
            "other voters.\nOpen gaps:\n" + listing + "\n\n" + _SELECT_QUESTION)


def _parse_pick(text: str, valid_ids: set) -> str | None:
    """Pull a valid plan id out of one voter's reply. Matches the LONGEST valid id that appears as a
    token so 'H-28-WIRE-1' wins over 'H-28'. No id found -> None (see _classify_vote for whether that
    is an explicit NONE abstention or an unparseable reply)."""
    if not text:
        return None
    for pid in sorted(valid_ids, key=len, reverse=True):
        if re.search(rf"(?<![\w-]){re.escape(pid)}(?![\w-])", text):
            return pid
    return None


def _classify_vote(text: str, valid_ids: set) -> tuple:
    """Classify ONE voter reply into ('id', pid) | ('none', None) | ('unparseable', None). An explicit
    NONE is a PARSEABLE abstention (the voter engaged with the ballot and said 'nothing actionable'),
    which is a different signal from an agentic derail / empty reply (unparseable). First-line answers
    win (the hardened prompt demands the pick be the FIRST LINE): a first line that is a valid id or a
    bare NONE decides even if the explanation below mentions other ids; otherwise fall back to an
    anywhere-in-text id match (longest wins), then an anywhere uppercase-NONE token."""
    if not text or not text.strip():
        return "unparseable", None
    first = text.strip().splitlines()[0].strip()
    pid = _parse_pick(first, valid_ids)
    if pid:
        return "id", pid
    if re.fullmatch(r"none[.!]?", first, re.IGNORECASE):
        return "none", None
    pid = _parse_pick(text, valid_ids)
    if pid:
        return "id", pid
    if re.search(r"\bNONE\b", text):
        return "none", None
    return "unparseable", None


def quorum_select(eligible: list, post_fn=quorum_triage.default_post) -> tuple:
    """Blind free-model vote over the TOP-K of the eligible pool. Returns (winner_id | None, detail).
    Fails CLOSED only when actionability is in doubt -- a halt beats a doubtful seed, because a bad
    pick here costs a PAID Opus boot (the opposite asymmetry from the wake gate). Semantics:
      * voters only ever see the top SELECT_BALLOT_MAX candidates (the pool is priority-ordered), so a
        wide near-equal backlog cannot split the reliable voters into a structural tie;
      * a PARSEABLE vote is a valid id pick OR an explicit NONE abstention (both prove the voter
        engaged with the ballot); an agentic derail / empty / rambling reply is unparseable;
      * zero parseable votes -> None, reason=NO_PICK (every voter unreachable/derailed);
      * fewer than SELECT_QUORUM_MIN parseable votes -> None, reason=QUORUM_THIN (a lone engaged voter
        must not decide a paid boot alone);
      * floor met but every parseable vote is NONE -> None, reason=ALL_NONE (genuine 'nothing
        actionable' stays fail-closed);
      * plurality + tie-break run over id-picks only: a clear leader -> reason=PICK; a TIE among
        leaders is NOT doubt about actionability (the tied voters each affirmed an actionable plan;
        they merely split preference) -> break by pool order among the tied leaders,
        reason=TIE_BROKEN, full tally+voters kept in detail."""
    ballot = eligible[:SELECT_BALLOT_MAX]
    valid = {g["id"] for g in ballot}
    prompt = _select_prompt(ballot)
    tally: dict = {}
    voters = []
    parseable = 0
    for model in quorum_triage.QUORUM_ROSTER:
        res = post_fn(model, prompt)
        if res.get("ok"):
            vote, pick = _classify_vote(res.get("text", ""), valid)
        else:
            vote, pick = "unparseable", None
        voters.append({"model": model, "pick": pick, "vote": vote, "status": res.get("status"),
                       "error": res.get("error"), "raw": (res.get("text") or "")[:200]})
        if vote != "unparseable":
            parseable += 1
        if pick:
            tally[pick] = tally.get(pick, 0) + 1
    if parseable == 0:
        return None, {"reason": "NO_PICK", "parseable": parseable, "tally": tally, "voters": voters}
    if parseable < SELECT_QUORUM_MIN:
        return None, {"reason": "QUORUM_THIN", "parseable": parseable, "tally": tally,
                      "voters": voters}
    if not tally:  # floor met, every engaged voter abstained -> genuinely nothing actionable
        return None, {"reason": "ALL_NONE", "parseable": parseable, "tally": tally, "voters": voters}
    top = max(tally.values())
    leaders = [pid for pid, n in tally.items() if n == top]
    if len(leaders) == 1:
        return leaders[0], {"reason": "PICK", "winner": leaders[0], "parseable": parseable,
                            "tally": tally, "voters": voters}
    # A split among affirmed leaders is preference, not doubt: break by pool order among the tied ids.
    order = {g["id"]: i for i, g in enumerate(ballot)}
    winner = min(leaders, key=lambda pid: order.get(pid, len(ballot)))
    return winner, {"reason": "TIE_BROKEN", "winner": winner, "parseable": parseable,
                    "tally": tally, "voters": voters}


def _build_pickup(gap: dict, eligible_ids: list, corpus_index: dict | None = None) -> str | None:
    """Build the pickup and reject any drift in its pinned audit evidence."""
    import hashlib

    def record_drift(kind, reason):
        report = gap.get("_audit_report")
        if not isinstance(report, dict):
            return
        exclusions = report.setdefault("exclusions", {})
        exclusions[kind] = exclusions.get(kind, 0) + 1
        entries = report.setdefault("excluded", [])
        item = {"id": gap.get("id"), "class": kind, "reason": reason}
        if item not in entries:
            entries.append(item)

    audit_path = gap.get("audit_evidence_path")
    audit_sha = gap.get("audit_evidence_sha256")
    citations = gap.get("audit_citations")
    if audit_path is not None or audit_sha is not None or citations is not None:
        if (not isinstance(audit_path, str) or not audit_path
                or not isinstance(audit_sha, str) or not audit_sha
                or not isinstance(citations, list) or not citations):
            record_drift("SCHEMA", "audit_pin_missing")
            return None
        try:
            evidence_raw = Path(audit_path).read_bytes()
        except OSError:
            record_drift("audit_evidence_drift", "audit_evidence_missing")
            return None
        if hashlib.sha256(evidence_raw).hexdigest() != audit_sha:
            record_drift("audit_evidence_drift", "audit_evidence_sha256_mismatch")
            return None
        for pin in citations:
            if not isinstance(pin, dict):
                record_drift("citation_source_drift", "citation_pin_not_object")
                return None
            source = pin.get("file")
            expected_sha = pin.get("sha256")
            line_number = pin.get("line")
            expected_text = pin.get("text")
            if (not isinstance(source, str) or not source
                    or not isinstance(expected_sha, str) or not expected_sha
                    or not isinstance(line_number, int) or isinstance(line_number, bool)
                    or line_number <= 0 or not isinstance(expected_text, str)):
                record_drift("citation_source_drift", "citation_pin_shape_invalid")
                return None
            try:
                source_raw = Path(source).read_bytes()
                actual_sha = hashlib.sha256(source_raw).hexdigest()
                lines = source_raw.splitlines(keepends=True)
                actual_raw = lines[line_number - 1]
                if actual_raw.endswith(b"\r\n"):
                    actual_raw = actual_raw[:-2]
                elif actual_raw.endswith(b"\n") or actual_raw.endswith(b"\r"):
                    actual_raw = actual_raw[:-1]
                actual_text = actual_raw.decode("utf-8")
            except (OSError, UnicodeDecodeError, IndexError):
                record_drift("citation_source_drift", "citation_source_unreadable")
                return None
            if actual_sha != expected_sha or actual_text != expected_text:
                record_drift("citation_source_drift", "citation_source_drift")
                return None

    files = ", ".join(gap["files"]) if gap["files"] else "(none named in the gap)"
    recon_note = ""
    if gap.get("recon_first"):
        recon_note = (
            "RECON-FIRST: this unit was enumerated from the plan SUMMARY index and is NOT "
            "audit-confirmed to have an existing production seam -- treat it as unaudited: classify "
            "executable-vs-greenfield in S1 BEFORE any write, and if the seam is greenfield/undesigned "
            "SET_ASIDE it (do not invent a seam). "
        )
    evidence_note = (
        f"audit_evidence={audit_path or '(none)'} "
        f"audit_evidence_sha256={audit_sha or '(none)'}"
    )
    base = (
        f"AUTO-SELECTED from the ELAI open-gaps deck by the free-gated plan_selector (no paid model "
        f"chose this). {recon_note}Gap {gap['id']}: {gap['desc']} "
        f"Referenced files (verified CLEAN of the /mnt/f dirty set at selection time): {files}. "
        f"{evidence_note}. "
        "fresh_recon=required. production_caller=glm_supervisor._try_autoseed. "
        "adversarial_tests=required. wire_check=required. plan_spec=required. land_evidence=required. "
        "Run the SKILL S1/S2/S3 recon to confirm the production seam EXISTS. If the seam is "
        "greenfield/undesigned (I-33 class), SET_ASIDE to escalation_queue -- do NOT invent a seam. "
        "If executable, route per the SKILL and the boot-prompt routing directive (while the CODEX-FIRST "
        "window is active, route the EXECUTOR to codex for EVERY executable unit, trivial or hard, to "
        "burn the paid credits before 2026-07-15; free swe lane is recon/fallback only), then land via "
        f"opus_land. Other eligible gaps this cycle: "
        f"{', '.join(i for i in eligible_ids if i != gap['id']) or '(none)'}."
    )
    if corpus_index is not None:
        try:
            import corpus_drain
            brief = corpus_drain.build_brief(gap["id"], corpus_index)
            if brief and "(not found in plan-index)" not in brief:
                base = base + "\n\n" + brief
        except Exception:
            pass
    return base


def select_next(state: dict, plan_status_dir: str, status_reader=read_status,
                dirty_reader=dirty_basenames, post_fn=quorum_triage.default_post,
                summary_dir: Path = SUMMARY_DIR, synthesis_path: Path | None = None,
                disp_reader=walk_state.dispositioned_ids, corpus_reader=None) -> dict | None:
    """Choose the next plan to seed, or None to let the supervisor halt. Fail-closed: unreadable/stale
    status, a fully-blind WIP guard (dirty_reader returns None: both git legs failed), or a doubtful
    quorum all return None. The candidate pool is the --punch head
    plus SUMMARY-index enumeration (build_pool), so a drained punch no longer halts the walk -- it
    refills from the wider backlog. Any id already carrying a terminal disposition (drained/escalated)
    in the teikoku-local ledger is excluded from the pool AND the eligible ids handed to the quorum
    (disp_reader). An empty pool (nothing anywhere) still returns None. On a pick returns
    {plan_id, pickup, detail}. status_reader/dirty_reader/post_fn/summary_dir/synthesis_path/disp_reader
    are injectable so tests never touch the network, git, the disposition ledger, or the F: tree."""
    status = status_reader()
    if not status:
        return None
    # Age gate SPLIT (deliverable 2): a stale snapshot DEMOTES the punch tier (a stale --punch head
    # must not drive a paid boot) but MUST NOT halt -- the corpus/summary tiers need no snapshot
    # freshness, so refill still runs. The old `return None` here disabled ALL refill on a stale punch.
    age = status.get("snapshot_age_min")
    punch_stale = isinstance(age, (int, float)) and age > AUTOSEED_MAX_SNAPSHOT_AGE_MIN
    if punch_stale:
        print(f"[plan_selector] punch demoted: snapshot stale (age={age} > "
              f"{AUTOSEED_MAX_SNAPSHOT_AGE_MIN}m); corpus/summary refill still runs", flush=True)
    dirty = dirty_reader()
    if dirty is None:  # both WIP-guard legs blind (git status AND stash failed) -> halt, don't seed
        print("[plan_selector] DIRTY_GUARD_BLIND: git status and stash legs both failed; cannot "
              "verify owner-WIP overlap -- halting (fail-closed) rather than seed a paid boot blind",
              flush=True)
        return None
    corpus_ordered, corpus_index = ([], None)
    if corpus_reader is not None:
        try:
            corpus_ordered, corpus_index = corpus_reader()
        except Exception:  # noqa: BLE001 - corpus tier additive; a failure just falls back
            corpus_ordered, corpus_index = [], None
    pool, stats = build_pool(status, state, dirty, plan_status_dir,
                             summary_dir=summary_dir, synthesis_path=synthesis_path,
                             dispositioned=disp_reader(), punch_stale=punch_stale,
                             corpus_ordered=corpus_ordered, corpus_index=corpus_index)
    print(f"[plan_selector] pool: punch={stats['punch']} corpus={stats.get('corpus', 0)} "
          f"summary={stats['summary']} "
          f"(audited_open={stats['audited_open']} recon_first={stats['recon_first']} of "
          f"{stats['summary_total_eligible']} eligible); dispositioned={stats['dispositioned']} "
          f"excluded_letters={stats['excluded_letters']}",
          flush=True)
    if not pool:
        return None
    # Ballot rotation (head-of-line fix): punch tier always leads the ballot; the corpus/summary tail
    # is rotated by the persisted offset so a stable-rejected head cannot starve the rest of the pool.
    punch_n = min(stats["punch"], SELECT_BALLOT_MAX)
    tail = pool[stats["punch"]:]
    offset = _rotation_offset() % len(tail) if tail else 0
    voting_pool = pool[:stats["punch"]] + tail[offset:] + tail[:offset]
    if offset:
        window = [g["id"] for g in voting_pool[:SELECT_BALLOT_MAX]]
        print(f"[plan_selector] ballot window rotated: offset={offset}/{len(tail)} "
              f"showing={window}", flush=True)
    winner_id, qdetail = quorum_select(voting_pool, post_fn)
    print(f"[plan_selector] quorum: reason={qdetail.get('reason')} "
          f"tally={qdetail.get('tally')}", flush=True)
    if qdetail.get("reason") == "ALL_NONE" and tail:
        new = _advance_rotation(SELECT_BALLOT_MAX - punch_n, len(tail))
        print(f"[plan_selector] ALL_NONE on this window -> rotation advanced to offset={new} "
              f"(next cycle votes the next {SELECT_BALLOT_MAX} units)", flush=True)
    if not winner_id:
        return None
    gap = next((g for g in pool if g["id"] == winner_id), None)
    if gap is None:
        return None
    eligible_ids = [g["id"] for g in pool]
    return {
        "plan_id": gap["id"],
        "pickup": _build_pickup(gap, eligible_ids, corpus_index=corpus_index),
        "detail": {"desc": gap["desc"], "files": gap["files"], "eligible": eligible_ids,
                   "recon_first": gap.get("recon_first", False), "snapshot_age_min": age,
                   "quorum": qdetail.get("reason"), "tally": qdetail.get("tally"),
                   "pool": stats},
    }
