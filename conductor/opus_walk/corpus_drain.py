#!/usr/bin/env python3
"""corpus_drain.py - full-corpus, supersession-aware, delivery-grounded drain engine.

Replaces the walk's capped-40 punch+SUMMARY frontier (plan_selector.build_pool) with a
complete ordered drain over the analyst CARD INDEX (workflow/reasoning/backlog-drain/cards-*.md).
Owns exactly one concern: parse the cards -> a plan-index -> classify every unit ->
supersession-collapse -> topo-ordered drain list + a per-plan super-summary brief. plan_selector
consumes the ordered list; the SUPERSESSION logic stays in this layer, not leaked into build_pool.

STAGE A classify: DONE | SKIP_SUPERSEDED | BUILD_RESIDUAL | BUILD | RECON.
  The load-bearing safety is the fail-closed CONFIRMED-SHIPPED guard on SKIP_SUPERSEDED: a dark
  unit skips ONLY if every id in its superseded_by is PROVEN SHIPPED (is_plan_done OR a drained
  disposition OR that superseder's own card state == wired). An unconfirmed superseder keeps the
  unit as BUILD -- the walk never skips real work on a superseder that never shipped.
STAGE B collapse: a partially-superseded unit builds only its residual (BUILD_RESIDUAL).
STAGE C order: kill-gates first (M gated behind K's M-0) -> proof-spine PS-1..PS-6 owners ->
  dependency (foundations before extensions) -> hubs (N-1 before N-3..N-12) -> PSR-1 / letter-number
  tiebreak. DONE + SKIP_SUPERSEDED are excluded from the build order (recorded, not built).

Stdlib only. Mirrors plan_selector's posture: named constants, defensive parsing (a garbled card
contributes fewer units + a parse_warnings row, never an exception), atomic writes, fail-safe live
signal reads (a missing plan-status / disposition ledger never raises).
"""
from __future__ import annotations

import glob
import json
import os
import re
import sys

try:
    import walk_state  # sibling: is_plan_done + dispositioned_ids + DISPOSITIONS_PATH
except Exception:  # pragma: no cover - standalone/test import fallback
    walk_state = None

# --- fixed wiring (named constants) ------------------------------------------
# The canonical ELAI workflow root on the teikoku distro (the /mnt/f share). Overridable for tests.
ELAI_WORKFLOW_ROOT = os.environ.get("ELAI_WORKFLOW_ROOT", "/workspace/ELAI")
# The analyst card index (one line/block per unit + a SUPERSESSION_EDGES block each).
CARDS_DIR = os.path.join(ELAI_WORKFLOW_ROOT, "workflow", "reasoning", "backlog-drain")
CARDS_GLOB = os.path.join(CARDS_DIR, "cards-*.md")
# The plan-index lives beside this module (walk-local) + on F: (feeder/core canonical).
LOCAL_INDEX_PATH = os.path.join(os.path.dirname(os.path.abspath(__file__)), "plan-index.json")
F_INDEX_PATH = os.path.join(ELAI_WORKFLOW_ROOT, "workflow", "status", "plan-index.json")
INDEX_SCHEMA_VERSION = 1

# Letter drain order (proof-spine + newest-last). A letter outside this string sorts last.
PLAN_LETTERS = "BCDEFGHIJKLMNOPQ"
# Proof-spine PS-1..PS-6 owner units (proof-spine.md): elevated ahead of the general backlog.
PS_OWNER_UNITS = ("G-6", "F-2b", "F-2", "K-4", "L-10", "K-2", "K-18")
# Kill-gate invocations (proof-spine PSR-1 tier 0) if any land as card units.
KILL_GATE_UNITS = ("A-1", "A-2", "A-3", "A-4", "K-0")
# N-1 is the single highest-fan-in unit (~30 promotions terminate there); it is the true unlock.
KEYSTONE_UNIT = "N-1"
# GATE M-0 blocks all of M behind K's kill-gates: M sorts after every non-M unit in its tier.
GATED_LETTERS = ("M",)

# A card's superseded_by/supersedes/respect field is "empty" when it is one of these tokens.
_NONE_TOKENS = {"", "-", "—", "none", "n/a", "na", "(none)", "(unbuilt)", "tbd"}

# A plan-unit id: LETTER(B..Q)-digit start, optional .frac, optional lowercase suffix, optional
# -TOKEN tails (H-28-WIRE-1). \b guards keep MD-1 / GD-1 / PS-1 / MA-1 / 4G / W-8 out (those have no
# [A-Q]-<digit> at a word boundary), so only real letter-plan ids are captured.
_ID_RE = re.compile(r"\b([A-Q]-\d+(?:\.\d+)?[a-z]?(?:-[A-Za-z0-9]+)*)\b")
# Leading id + (letter, number) sort key parse.
_IDNUM_RE = re.compile(r"^([A-Q])-(\d+)(?:\.(\d+))?([a-z])?")

# Field markers: anchored to line-start / a `|` delimiter (single-line EFGH/IJKL cards) OR, for the
# bracket forms only (MNOPQ dual-field lines like `supersedes[]: x  superseded_by[]: y`), any
# whitespace boundary. The bracket disambiguates the mid-line case so prose never false-triggers.
_FIELD_RE = re.compile(
    r"(?:(?<=\|)|(?<=\n)|^)\s*"
    r"(does|state|supersedes|superseded_by|residual|key_files|respect|complements)(?:\[\])?\s*:"
    r"|(?<=\s)(supersedes|superseded_by|residual|key_files|respect|complements|does|state)\[\]\s*:",
    re.M,
)

# Relation keywords, and the subset that is a GENUINE supersession (older is owned/closed/replaced by
# newer) vs merely additive EXTENDS (older stays valid). Only genuine relations feed superseded_by.
_GENUINE_RELATIONS = (
    "SUPERSED", "REPLAC", "CLOSE", "CLOSED", "ABSORB", "SUBSUM", "AMEND",
    "DEMOTE", "RETIRE", "UNIF", "GENERALIZ", "COMPLETE", "ACTIVATE", "FUSE", "CORRECT",
)
_ALL_RELATIONS = _GENUINE_RELATIONS + ("EXTEND", "COMPLEMENT", "REFUTE", "MAP", "TYPE", "WIRE", "BUILD")
# An "<id> <relation-word>" edge head, used to split multi-edge lines into per-edge clauses.
_EDGE_HEAD_RE = re.compile(
    r"([A-Q]-\d+(?:\.\d+)?[a-z]?(?:-[A-Za-z0-9]+)*)\s+"
    r"(?:hard-|scope-|behavior-)?([A-Za-z]*(?:%s)[A-Za-z/-]*)" % "|".join(_ALL_RELATIONS),
    re.I,
)


# =============================================================================
# PARSER  (4 analyst formats -> one plan-index)
# =============================================================================

def _match_header(line):
    """Return (unit_id, style) for a card-header line, else (None, None).
    style 'line' = single-line pipe-delimited card (EFGH `--- E-1 ... |`, IJKL `id: I-1 | ...`);
    style 'block' = multi-line card (BCD `## B-1 ...`, MNOPQ `id: M-1  title`)."""
    m = re.match(r"^##\s+([A-Q]-\d[\w.\-]*)", line)
    if m:
        return m.group(1).rstrip(":.,"), "block"
    m = re.match(r"^---\s+([A-Q]-\d[\w.\-]*)\b", line)
    if m and "|" in line:
        return m.group(1).rstrip(":.,"), "line"
    m = re.match(r"^id:\s*([A-Q]-\d[\w.\-]*)", line)
    if m:
        rest = line[m.end():].lstrip()
        return m.group(1).rstrip(":.,"), ("line" if rest.startswith("|") else "block")
    return None, None


def _is_separator(line):
    """A section-separator / non-card structural line that terminates a multi-line block."""
    s = line.strip()
    if not s:
        return False
    if re.match(r"^={3,}$", s) or re.match(r"^-{4,}", s):
        return True
    up = s.upper()
    return (up.startswith("END ") or up.startswith("SUPERSESSION_EDGES")
            or up.startswith("DRAIN-SYSTEM") or up.startswith("CARDS")
            or re.match(r"^-{2,}\s*[A-Z].*(PLAN|SCOPE|CARDS)", up) is not None)


def _extract_fields(block_text):
    """Split a card block (single line or multi-line) into {field: value} + the header prefix
    (everything before the first field marker)."""
    marks = []
    for m in _FIELD_RE.finditer(block_text):
        key = m.group(1) or m.group(2)
        marks.append((m.start(), m.end(), key))
    if not marks:
        return block_text, {}
    header = block_text[: marks[0][0]]
    fields = {}
    for i, (_ms, me, key) in enumerate(marks):
        end = marks[i + 1][0] if i + 1 < len(marks) else len(block_text)
        val = block_text[me:end].strip()
        # A single-line card's last field can trail a stray `|`; a pipe never belongs in a value.
        val = val.strip().strip("|").strip()
        if key not in fields:  # first occurrence wins (defensive against a stray prose match)
            fields[key] = val
    return header, fields


def _extract_ids(text, exclude=None):
    """All plan-unit ids in a supersedes/superseded_by field value, de-duped, order-preserved,
    minus the unit's own id. A none-token / empty value -> []."""
    if not text:
        return []
    if text.strip().lower() in _NONE_TOKENS:
        return []
    out, seen = [], set()
    for m in _ID_RE.finditer(text):
        cid = m.group(1)
        if cid == exclude or cid in seen:
            continue
        seen.add(cid)
        out.append(cid)
    return out


def _split_list(text):
    """A key_files/respect field -> a list of trimmed entries (comma/semicolon split), none-tokens
    and '(planned)'/'(unbuilt)' markers dropped. Empty -> []."""
    if not text or text.strip().lower() in _NONE_TOKENS:
        return []
    parts = re.split(r"[;,]", text)
    out = []
    for p in parts:
        p = p.strip().strip(".")
        if not p or p.lower() in _NONE_TOKENS:
            continue
        out.append(p)
    return out


def _normalize_state(text):
    """Map a free-text state value to one canonical token: wired|partial|dark|uncertain|refuted.
    Fail-safe toward MORE work: any mention of 'partial' wins over dark/wired (never skip a unit that
    has some shipped legs); a bare unknown state -> 'uncertain' (routes to RECON, never a silent skip)."""
    if not text:
        return "uncertain"
    seg = re.split(r"[(—.,]", text, 1)[0].lower()  # up to first ( , . or em-dash
    if "refut" in seg:
        return "refuted"
    if "uncertain" in seg:
        return "uncertain"
    if "partial" in seg:
        return "partial"
    if "not_start" in seg or "not start" in seg or "not-start" in seg:
        return "dark"
    if "dark" in seg:  # includes wire_dark / wire-dark / shipped_wire_dark
        return "dark"
    if re.search(r"wired", seg):
        return "wired"
    return "uncertain"


def _parse_edge_region(text, warnings):
    """Best-effort parse of a SUPERSESSION_EDGES block into {newer, older, evidence, relation}
    rows. Splits each line at every '<id> <relation>' head so multi-edge lines are handled; the
    olders are the ids between one relation and the next head (or the evidence delimiter). Never
    raises: a line it cannot split is skipped + counted, never fatal."""
    edges = []
    for raw in text.splitlines():
        line = raw.strip()
        if not line or line.startswith("#") or _is_separator(line):
            continue
        heads = list(_EDGE_HEAD_RE.finditer(line))
        if not heads:
            continue
        for i, hm in enumerate(heads):
            newer = hm.group(1)
            relation = hm.group(2).upper()
            seg_end = heads[i + 1].start() if i + 1 < len(heads) else len(line)
            segment = line[hm.end():seg_end]
            # Trim the evidence tail (starts at the first — / (" / : after the relation).
            cut = re.search(r"\s[—(:]|\s\"", segment)
            olders_text = segment[: cut.start()] if cut else segment
            olders = _extract_ids(olders_text, exclude=newer)
            if not olders:
                warnings.append(f"edge with no resolvable older id: {line[:100]!r}")
                continue
            evidence = line[hm.start():][:160]
            genuine = any(r in relation for r in _GENUINE_RELATIONS) and "EXTEND" not in relation
            for older in olders:
                edges.append({"newer": newer, "older": older,
                              "evidence": evidence, "relation": relation, "genuine": genuine})
    return edges


def parse_cards(cards_glob=CARDS_GLOB):
    """Read every cards-*.md file into {units_by_id, edges, warnings}. Defensive: a missing/garbled
    file contributes fewer units and a parse_warnings row, never an exception (mirrors
    summary_candidates posture). Duplicate ids across files keep the first + warn."""
    warnings = []
    units = {}
    edges = []
    files = sorted(glob.glob(cards_glob))
    if not files:
        warnings.append(f"no card files matched {cards_glob}")
    for path in files:
        try:
            with open(path, "r", encoding="utf-8", errors="replace") as fh:
                text = fh.read()
        except OSError as exc:
            warnings.append(f"unreadable card file {os.path.basename(path)}: {exc}")
            continue
        lines = text.splitlines()

        # Split the file into an edge region (SUPERSESSION_EDGES .. first card header) + card blocks.
        first_header_idx = None
        header_idx = []
        for i, ln in enumerate(lines):
            uid, _style = _match_header(ln)
            if uid is not None:
                header_idx.append(i)
                if first_header_idx is None:
                    first_header_idx = i
        edge_region_text = ""
        for i, ln in enumerate(lines[: first_header_idx if first_header_idx is not None else len(lines)]):
            if "SUPERSESSION_EDGES" in ln.upper():
                edge_region_text = "\n".join(lines[i:first_header_idx])
                break
        edges.extend(_parse_edge_region(edge_region_text, warnings))

        # Card blocks.
        for n, start in enumerate(header_idx):
            uid, style = _match_header(lines[start])
            if style == "line":
                block = lines[start]
            else:
                # gather to the next header or a structural separator
                end = header_idx[n + 1] if n + 1 < len(header_idx) else len(lines)
                body = []
                for j in range(start, end):
                    if j > start and _is_separator(lines[j]):
                        break
                    body.append(lines[j])
                block = "\n".join(body)

            _header, fields = _extract_fields(block)
            if "state" not in fields:
                warnings.append(f"{uid}: no state field parsed (card {os.path.basename(path)})")
            unit = {
                "id": uid,
                "does": (fields.get("does") or "").strip(),
                "state": _normalize_state(fields.get("state") or ""),
                "supersedes": _extract_ids(fields.get("supersedes"), exclude=uid),
                "superseded_by": _extract_ids(fields.get("superseded_by"), exclude=uid),
                "residual": (fields.get("residual") or "").strip(),
                "key_files": _split_list(fields.get("key_files")),
                "respect": _split_list(fields.get("respect")),
            }
            if not unit["does"]:
                warnings.append(f"{uid}: no does field parsed (card {os.path.basename(path)})")
            if uid in units:
                warnings.append(f"duplicate unit id {uid} (kept first; {os.path.basename(path)})")
                continue
            units[uid] = unit

    # Union GENUINE supersession edges into the per-unit graph (per-card fields stay authoritative;
    # edges only enrich). EXTENDS edges are additive and never populate superseded_by.
    for e in edges:
        if not e["genuine"]:
            continue
        newer, older = e["newer"], e["older"]
        if older in units and newer not in units[older]["superseded_by"] and newer != older:
            units[older]["superseded_by"].append(newer)
        if newer in units and older not in units[newer]["supersedes"] and newer != older:
            units[newer]["supersedes"].append(older)

    return {"units": units, "edges": edges, "warnings": warnings}


def build_index(cards_glob=CARDS_GLOB):
    """parse_cards -> the canonical plan-index dict (schema_version/units[]/edges[]/parse_warnings)."""
    parsed = parse_cards(cards_glob)
    units = [parsed["units"][k] for k in sorted(parsed["units"], key=_sort_key_id)]
    edges = [{"newer": e["newer"], "older": e["older"], "evidence": e["evidence"]}
             for e in parsed["edges"]]
    # dedupe edges on (newer, older)
    seen, deduped = set(), []
    for e in edges:
        k = (e["newer"], e["older"])
        if k in seen:
            continue
        seen.add(k)
        deduped.append(e)
    return {
        "schema_version": INDEX_SCHEMA_VERSION,
        "units": units,
        "edges": deduped,
        "parse_warnings": parsed["warnings"],
    }


def _atomic_write_json(path, data):
    directory = os.path.dirname(path) or "."
    os.makedirs(directory, exist_ok=True)
    tmp = f"{path}.{os.getpid()}.tmp"
    try:
        with open(tmp, "w", encoding="utf-8") as fh:
            json.dump(data, fh, indent=2)
        os.replace(tmp, path)
    except BaseException:
        try:
            os.remove(tmp)
        except OSError:
            pass
        raise


def write_index(index, local_path=LOCAL_INDEX_PATH, f_path=F_INDEX_PATH):
    """Write the plan-index to the walk-local path and (best-effort) the F: canonical path.
    A failure to reach /mnt/f never blocks the local write (the walk reads the local copy)."""
    _atomic_write_json(local_path, index)
    written = [local_path]
    try:
        _atomic_write_json(f_path, index)
        written.append(f_path)
    except OSError as exc:
        print(f"[corpus_drain] WARN: could not write F: canonical index {f_path}: {exc}",
              file=sys.stderr)
    return written


def load_index(path=LOCAL_INDEX_PATH):
    """Read the plan-index.json. Fail-safe: if the file is absent it is built in-memory from the
    cards (so a dry-run works before the first --build-index); a corrupt file raises ValueError."""
    if not os.path.isfile(path):
        return build_index()
    with open(path, "r", encoding="utf-8") as fh:
        data = json.load(fh)
    if not isinstance(data, dict) or "units" not in data:
        raise ValueError(f"plan-index at {path} is malformed (no units)")
    return data


# =============================================================================
# LIVE DELIVERY SIGNAL  (is_shipped_fn default)
# =============================================================================

def _drained_ids():
    """Plan ids with accepted DONE proof, not merely a review-settled disposition.

    Historical ``drained`` rows often have no landed commit.  Those rows prevent
    reseeding, but they are claims and must not remove work from the drain.
    """
    path = getattr(walk_state, "DISPOSITIONS_PATH", None) if walk_state else None
    if not path or not os.path.isfile(path):
        return set()
    try:
        with open(path, "r", encoding="utf-8") as fh:
            data = json.load(fh)
    except (OSError, ValueError):
        return set()
    disp = data.get("dispositions") if isinstance(data, dict) else None
    if not isinstance(disp, dict):
        return set()
    proven = set()
    for pid, row in disp.items():
        if not isinstance(row, dict) or row.get("verdict") not in ("drained", "verified_delivered"):
            continue
        if walk_state is None:
            continue
        try:
            if walk_state.is_plan_done(pid):
                proven.add(pid)
        except Exception:
            continue
    return proven


def default_is_shipped(unit_id):
    """Proof-backed shipped signal.  A no-commit drained claim is never enough."""
    if walk_state is not None:
        try:
            if walk_state.is_plan_done(unit_id):
                return True
        except Exception:
            pass
    return False


# =============================================================================
# STAGE A: CLASSIFY
# =============================================================================

DONE = "DONE"
SKIP_SUPERSEDED = "SKIP_SUPERSEDED"
BUILD_RESIDUAL = "BUILD_RESIDUAL"
BUILD = "BUILD"
RECON = "RECON"

# Actions excluded from the build order (recorded, never popped for a paid turn).
_TERMINAL_ACTIONS = (DONE, SKIP_SUPERSEDED)


def _residual_material(residual):
    """True when the residual names concrete remaining work (not none/empty)."""
    r = (residual or "").strip().lower().rstrip(".")
    if not r or r in _NONE_TOKENS:
        return False
    return not r.startswith("none")


def _units_by_id(index):
    return {u["id"]: u for u in index.get("units", [])}


def _superseder_confirmed_shipped(sid, by_id, is_shipped_fn):
    """CONFIRMED-SHIPPED (fail-closed): a superseder S is confirmed iff is_shipped_fn(S) OR S's own
    card state == wired. An unconfirmed / dark / absent S is NOT confirmed -> the superseded unit is
    never skipped. This is the load-bearing guard of verify-then-skip."""
    if is_shipped_fn(sid):
        return True
    su = by_id.get(sid)
    return bool(su and su.get("state") == "wired")


def classify(unit, index, is_shipped_fn=default_is_shipped, by_id=None):
    """One of DONE|SKIP_SUPERSEDED|BUILD_RESIDUAL|BUILD|RECON for a single unit.

    DONE            = live-shipped OR (state wired AND no material residual) OR refuted (terminal).
    SKIP_SUPERSEDED = state dark AND superseded AND EVERY superseder proven shipped (fail-closed).
    BUILD_RESIDUAL  = state partial (build only the residual), OR wired-with-a-material-residual.
    BUILD           = state dark, not superseded OR with an UNCONFIRMED superseder (fail-closed:
                      build the whole unit rather than skip real work).
    RECON           = state uncertain (settle before it becomes a build candidate).
    """
    if by_id is None:
        by_id = _units_by_id(index)
    uid = unit["id"]
    if is_shipped_fn(uid):
        return DONE
    state = unit.get("state", "uncertain")
    if state == "refuted":
        return DONE  # ran + closed dead; excluded from the build order, downstream forbidden
    if state == "uncertain":
        return RECON
    if state == "wired":
        return DONE if not _residual_material(unit.get("residual")) else BUILD_RESIDUAL
    if state == "partial":
        return BUILD_RESIDUAL  # some legs shipped -> build ONLY the residual (never skip)
    # state == 'dark'
    supers = unit.get("superseded_by") or []
    if supers and all(_superseder_confirmed_shipped(s, by_id, is_shipped_fn) for s in supers):
        return SKIP_SUPERSEDED
    return BUILD  # not superseded, or a superseder is unconfirmed -> fail-closed BUILD


def classify_all(index, is_shipped_fn=default_is_shipped):
    """{unit_id: action} for every unit in the index."""
    by_id = _units_by_id(index)
    return {u["id"]: classify(u, index, is_shipped_fn, by_id=by_id) for u in index.get("units", [])}


# =============================================================================
# STAGE C: ORDER
# =============================================================================

def _sort_key_id(uid):
    """(letter_index, numeric) for stable letter/number ordering. Unknown -> sorts last."""
    m = _IDNUM_RE.match(uid)
    if not m:
        return (99, 9999.0)
    letter, n, frac, suf = m.group(1), int(m.group(2)), m.group(3), m.group(4)
    num = n + (int(frac) / 1000.0 if frac else 0.0) + ((ord(suf) - 96) / 100.0 if suf else 0.0)
    li = PLAN_LETTERS.find(letter)
    return (li if li >= 0 else 99, num)


def _psr_tier(uid):
    """Proof-spine PSR-1 precedence tier (lower = earlier): 0 kill-gate, 1 N-1 substrate,
    2 PS-owner, 3 everything else."""
    if uid in KILL_GATE_UNITS:
        return 0
    if uid == KEYSTONE_UNIT:
        return 1
    if uid in PS_OWNER_UNITS:
        return 2
    return 3


def _dep_depth(uid, build_ids, by_id, _memo, _stack=None):
    """Longest foundation chain under uid within the build set: a unit U extends/supersedes its
    foundations (U.supersedes); a foundation still in the build set lands BEFORE U. Cycle-safe:
    a back-edge is broken (depth 0 for the revisited node) and the walk continues."""
    if uid in _memo:
        return _memo[uid]
    _stack = _stack or set()
    if uid in _stack:  # cycle: break at the lower-letter unit (return 0), caller logs at build time
        return 0
    _stack.add(uid)
    u = by_id.get(uid)
    preds = [p for p in (u.get("supersedes") if u else []) if p in build_ids]
    depth = 0 if not preds else 1 + max(_dep_depth(p, build_ids, by_id, _memo, _stack) for p in preds)
    _stack.discard(uid)
    _memo[uid] = depth
    return depth


def order(index, is_shipped_fn=default_is_shipped):
    """The drain ORDER: a list of unit ids to build, DONE + SKIP_SUPERSEDED excluded.
    Sort key = (psr_tier, dep_depth, gated_penalty, letter, number, id) -- kill-gates then N-1 then
    PS owners then dependency depth (foundations first), M pushed behind its M-0 gate, then a stable
    letter/number tiebreak. Defensive: incomplete dependency data degrades to (tier, letter, number),
    never crashes."""
    by_id = _units_by_id(index)
    actions = classify_all(index, is_shipped_fn)
    build_ids = {uid for uid, a in actions.items() if a not in _TERMINAL_ACTIONS}
    memo = {}

    def _key(uid):
        li, num = _sort_key_id(uid)
        letter = uid.split("-", 1)[0]
        gated = 1 if letter in GATED_LETTERS else 0
        try:
            depth = _dep_depth(uid, build_ids, by_id, memo)
        except Exception:
            depth = 0
        return (_psr_tier(uid), depth, gated, li, num, uid)

    return sorted(build_ids, key=_key)


def ordered_drain(index=None, is_shipped_fn=default_is_shipped):
    """Convenience wrapper: (ordered_build_ids, {id: action}). Loads the index if not supplied."""
    if index is None:
        index = load_index()
    return order(index, is_shipped_fn), classify_all(index, is_shipped_fn)


# =============================================================================
# PER-PLAN SUPER-SUMMARY BRIEF
# =============================================================================

def build_brief(unit_id, index, is_shipped_fn=default_is_shipped):
    """A compact per-plan brief the orchestrator reads at plan-start: the unit's card (does/state/
    residual/key_files) + a RESPECT / DO-NOT-REVERT section listing its respect[] entries AND any
    unit that supersedes it that is ALREADY SHIPPED (read their state, never re-author/regress their
    seam). Returns a plain string; a missing unit yields an explicit not-found line (never raises)."""
    by_id = _units_by_id(index)
    unit = by_id.get(unit_id)
    if unit is None:
        return f"DRAIN CARD {unit_id}: (not found in plan-index)"
    action = classify(unit, index, is_shipped_fn, by_id=by_id)
    lines = [
        f"DRAIN CARD {unit['id']}: ACTION={action}",
        f"does: {unit.get('does') or '(none)'}",
        f"state: {unit.get('state')}",
        f"RESIDUAL (build ONLY this): {unit.get('residual') or '(none)'}",
        f"key_files: {', '.join(unit.get('key_files') or []) or '(none named)'}",
        "RESPECT / DO-NOT-REVERT (these newer units own or extend this seam -- read their state,",
        "never re-author or regress their path):",
    ]
    respect_lines = []
    for r in unit.get("respect") or []:
        respect_lines.append(f"  - {r}")
    for sid in unit.get("superseded_by") or []:
        if not is_shipped_fn(sid) and not (by_id.get(sid) and by_id[sid].get("state") == "wired"):
            continue  # only SHIPPED superseders are hard don't-revert anchors
        su = by_id.get(sid)
        does = (su.get("does") if su else "") or ""
        st = (su.get("state") if su else "") or "shipped"
        respect_lines.append(f"  - {sid} [SHIPPED, {st}]: {does[:120]}")
    if not respect_lines:
        respect_lines.append("  - (no respect entries or shipped superseders)")
    lines.extend(respect_lines)
    supersedes = unit.get("supersedes") or []
    if supersedes:
        lines.append(f"SUPERSEDES (older units this one owns; consolidate, do not duplicate): "
                     f"{', '.join(supersedes)}")
    return "\n".join(lines)


# =============================================================================
# CLI
# =============================================================================

def _cmd_build_index():
    index = build_index()
    written = write_index(index)
    print(f"[corpus_drain] built plan-index: {len(index['units'])} units, "
          f"{len(index['edges'])} edges, {len(index['parse_warnings'])} parse_warnings")
    for p in written:
        print(f"  wrote {p}")
    if index["parse_warnings"]:
        print("  first parse_warnings:")
        for w in index["parse_warnings"][:10]:
            print(f"    - {w}")
    return index


def _cmd_dry_run():
    index = load_index()
    is_shipped = default_is_shipped
    by_id = _units_by_id(index)
    actions = classify_all(index, is_shipped)
    order_ids = order(index, is_shipped)

    total = len(index["units"])
    counts = {DONE: 0, SKIP_SUPERSEDED: 0, BUILD_RESIDUAL: 0, BUILD: 0, RECON: 0}
    for a in actions.values():
        counts[a] = counts.get(a, 0) + 1

    print("=" * 70)
    print("corpus_drain DRY-RUN (no build, no disposition write)")
    print("=" * 70)
    print(f"total units:        {total}")
    print(f"parse_warnings:     {len(index.get('parse_warnings', []))}")
    print("classification:")
    for k in (DONE, SKIP_SUPERSEDED, BUILD_RESIDUAL, BUILD, RECON):
        print(f"  {k:<16} {counts.get(k, 0)}")
    print(f"build order length: {len(order_ids)}")
    print("first 40 of the computed drain order:")
    print("  " + ", ".join(order_ids[:40]))

    # GUARD-CHECK: no SKIP_SUPERSEDED unit may have an unconfirmed superseder (fail-closed proof).
    wrongly_skipped = []
    for uid, a in actions.items():
        if a != SKIP_SUPERSEDED:
            continue
        supers = by_id[uid].get("superseded_by") or []
        if not supers or not all(_superseder_confirmed_shipped(s, by_id, is_shipped) for s in supers):
            wrongly_skipped.append(uid)
    # dark units held BACK from SKIP because a superseder is not proven-shipped (guard working).
    guard_held = []
    for uid, a in actions.items():
        if a != BUILD:
            continue
        u = by_id[uid]
        if u.get("state") == "dark" and (u.get("superseded_by") or []):
            guard_held.append(uid)
    print("-" * 70)
    print("FAIL-CLOSED GUARD CHECK:")
    print(f"  SKIP_SUPERSEDED units with an UNCONFIRMED superseder (MUST be 0): {len(wrongly_skipped)}")
    if wrongly_skipped:
        print("  !! GUARD BREACH: " + ", ".join(wrongly_skipped))
    print(f"  dark+superseded units correctly held as BUILD (unconfirmed superseder): {len(guard_held)}")
    if guard_held:
        print("    " + ", ".join(guard_held[:30]) + (" ..." if len(guard_held) > 30 else ""))
    print("=" * 70)
    return 0


def main(argv=None):
    argv = list(sys.argv[1:] if argv is None else argv)
    if "--build-index" in argv:
        _cmd_build_index()
        return 0
    if "--dry-run" in argv or not argv:
        return _cmd_dry_run()
    print("usage: corpus_drain.py [--build-index | --dry-run]", file=sys.stderr)
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
