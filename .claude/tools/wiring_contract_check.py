#!/usr/bin/env python3
"""wiring_contract_check.py - verify a B-N WIRING_CONTRACT block (FAIL-CLOSED).

The done-gate for a shipped feature. Fail-closed by default per
.claude/rules/wiring-enforcement.md (rules R1-R9): missing evidence is FAIL, the
runtime trigger runs unconditionally, the off-agent SSB row is the load-bearing
proof, db-not-found is FAIL (not UNVERIFIED), an incomplete contract is FAIL, and
a contract block that parses empty is FAIL (ArchUnit failOnEmptyShould).

CLI:
  python wiring_contract_check.py --bn-file PATH [--map PATH] [--project-root PATH]
                                  [--json] [--timeout 60]
                                  [--static-only] [--allow-missing-contract]
  --static-only            debug: skip the runtime trigger. A contract that
                           declares observable_effect can then never be PASS.
  --allow-missing-contract debug: a file with NO contract block returns SKIP
                           instead of FAIL (for incidental non-feature scans).
                           Never relaxes a present-but-incomplete contract.
  --run-trigger            deprecated no-op: the trigger always runs (R3).
Env overrides (portability): ELAI_MAP, ELAI_PROJECT_ROOT, ELAI_SSB_DB.
Exit: 0 PASS, 1 FAIL (incl. missing/incomplete contract), 2 SKIP (only with
--allow-missing-contract and no contract block).
"""
from __future__ import annotations

import argparse
import json
import os
import re
import sqlite3
import subprocess
import sys
import time
from fnmatch import fnmatch
from pathlib import Path
from typing import Any, Dict, List, Optional, Set

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
if hasattr(sys.stderr, "reconfigure"):
    sys.stderr.reconfigure(encoding="utf-8", errors="replace")

DEFAULT_MAP = os.environ.get("ELAI_MAP", r"F:\.ELAI_workflow\.claude\tools\elai.map")
DEFAULT_PROJECT_ROOT = os.environ.get(
    "ELAI_PROJECT_ROOT", r"F:\.ELAI_workflow\.elai_cc")
# Off-agent evidence db (R2/R4). ELAI_SSB_DB env override wins; otherwise resolved
# relative to the project root. The real SSB lives at <root>/sessions/ssb.db; the
# previous code looked at <root>/.elai/ssb.db and globbed
# <root>/.elai_cc/sessions/<entry>/ssb.db -- BOTH wrong, which made expect_ssb a
# dead no-op that always returned UNVERIFIED (a silent pass). See _find_ssb_db.
SSB_DB_ENV = os.environ.get("ELAI_SSB_DB")
SKIP_DIRS = {"target", "node_modules", ".bkp_old", "__pycache__"}
SCAN_EXTS = (".rs", ".toml", ".ts", ".py", ".svelte", ".json", ".yaml", ".yml")


def grep_map(pat: str, mp: str) -> List[str]:
    env = dict(os.environ); env.setdefault("LC_ALL", "C.UTF-8")
    try:
        p = subprocess.run(["grep", "-P", pat, mp], capture_output=True,
                           text=True, encoding="utf-8", errors="replace", env=env)
    except FileNotFoundError:
        return _py_grep(pat, mp)
    if p.returncode == 1:
        return []
    if p.returncode != 0:
        return _py_grep(pat, mp)
    return [ln for ln in p.stdout.splitlines() if ln]


def _py_grep(pat: str, where: str) -> List[str]:
    rx = re.compile(pat); out: List[str] = []
    if not os.path.isfile(where):
        return out
    try:
        with open(where, "r", encoding="utf-8", errors="replace") as fh:
            for ln in fh:
                if rx.search(ln):
                    out.append(ln.rstrip("\n"))
    except OSError:
        pass
    return out


def extract_block(file_path: str, heading: str = "WIRING_CONTRACT") -> Optional[str]:
    try:
        text = Path(file_path).read_text(encoding="utf-8", errors="replace")
    except OSError as e:
        print(f"ERROR reading {file_path}: {e}", file=sys.stderr); return None
    lines = text.splitlines(); start = None
    # word-boundary, not end-anchored: real contracts decorate the heading, e.g.
    # "## WIRING_CONTRACT (per Layer 0 of testing-trust-design.txt)". The old
    # end-anchored regex missed those -> block None. With missing-block now FAIL
    # (R1), an over-strict match would false-FAIL every real decorated contract.
    for i, ln in enumerate(lines):
        if re.match(rf"^##\s+{re.escape(heading)}\b", ln):
            start = i + 1; break
    if start is None:
        return None
    end = len(lines)
    for j in range(start, len(lines)):
        if re.match(r"^##\s+", lines[j]):
            end = j; break
    return "\n".join(lines[start:end])


def _strip_q(s: str) -> str:
    if (s.startswith('"') and s.endswith('"')) or (s.startswith("'") and s.endswith("'")):
        return s[1:-1]
    return s


def parse_loose_yaml(block: str) -> Dict[str, Any]:
    """key:, '  - x' lists. observable_effect handled separately."""
    result: Dict[str, Any] = {}; cur_list: Optional[List[Any]] = None
    for raw in block.splitlines():
        if not raw.strip() or raw.lstrip().startswith("#"):
            continue
        stripped = raw.lstrip(); indent = len(raw) - len(stripped)
        m_item = re.match(r"-\s+(.*)$", stripped)
        m_kv = re.match(r"([A-Za-z_][\w\-]*)\s*:\s*(.*)$", stripped)
        if m_item and cur_list is not None and indent > 0:
            cur_list.append(_strip_q(m_item.group(1).strip())); continue
        if m_kv:
            k, v = m_kv.group(1), m_kv.group(2).strip()
            if not v:
                lst: List[Any] = []; result[k] = lst; cur_list = lst
            elif v.lower() in ("null", "none", "~"):
                result[k] = None; cur_list = None
            else:
                result[k] = _strip_q(v); cur_list = None
    oe = _parse_observable(block)
    if oe:
        result["observable_effect"] = oe
    return result


def _parse_observable(block: str) -> Dict[str, str]:
    out: Dict[str, str] = {}; inside = False; base = -1
    for raw in block.splitlines():
        if not raw.strip():
            continue
        ind = len(raw) - len(raw.lstrip()); st = raw.strip()
        if st.lower().startswith("observable_effect:"):
            inside = True; base = ind
            tail = st.split(":", 1)[1].strip()
            if tail:
                return {"raw": _strip_q(tail)}
            continue
        if inside:
            if ind <= base:
                break
            m = re.match(r"([A-Za-z_][\w\-]*)\s*:\s*(.*)$", st)
            if m:
                out[m.group(1)] = _strip_q(m.group(2).strip())
    return out


# elai.map stores BARE symbol names in the SYMBOL column (col 2), not the
# fully-qualified "crates/.../file.rs::Symbol" path the WIRING_CONTRACT writes.
# A full-path grep therefore always returns 0 hits on real symbols. We resolve
# the bare leaf (last component after :: or /) as a fallback. File-path-shaped
# produces (a .toml/.svelte/etc with no ::) are checked on the filesystem, since
# the mapper emits DEF rows for symbols, not for whole-file artifacts.
_PATH_EXTS = (".toml", ".svelte", ".json", ".yaml", ".yml", ".md", ".css")


def _leaf_symbol(s: str) -> str:
    """Bare leaf for a contract symbol: drop the file:: prefix, take last :: arm."""
    leaf = s.split("::")[-1].strip() if "::" in s else s
    # a leaf like "for_pre_tool_use" from "HookInvocation::for_pre_tool_use" is
    # already bare; a path-only entry has no :: so leaf == s, handled by caller.
    if "/" in leaf and "::" not in s:
        leaf = leaf.rsplit("/", 1)[-1]
    return leaf


def _is_path_produces(s: str) -> bool:
    """True for whole-file artifacts (e.g. crates/.../sqli.toml) with no symbol."""
    return "::" not in s and (s.endswith(_PATH_EXTS) or "/" in s and "." in s.rsplit("/", 1)[-1])


def _resolve_produces_path(s: str, project_root: str) -> Optional[str]:
    """Locate a file-path produces under project_root (rel path or trailing match)."""
    rel = s.replace("\\", "/").lstrip("/")
    cand = os.path.join(project_root, rel.replace("/", os.sep))
    if os.path.isfile(cand):
        return cand
    base = os.path.basename(rel)
    for r, dirs, files in os.walk(project_root):
        dirs[:] = [d for d in dirs if d not in SKIP_DIRS]
        if base in files:
            full = os.path.join(r, base)
            if full.replace("\\", "/").endswith(rel):
                return full
    return None


def _def_hits(symbol: str, mp: str) -> List[str]:
    """Tab-bounded grep for a bare symbol; returns rows (any tag)."""
    return grep_map(rf"\t{re.escape(symbol)}\t", mp)


def _file_hint(sym: str) -> Optional[str]:
    """For an F-plan-form contract `crates/.../file.rs::Symbol`, return the
    forward-slashed source path (`crates/.../file.rs`). Module-path-form
    contracts (`orchestrator::module::Type::leaf`, no source-file prefix) and
    bare symbols return None — those keep the unconstrained module-path leaf
    behaviour. The file lock is what stops a same-named DEF in a DIFFERENT file
    from satisfying a produces whose contract path is wrong (the E-12
    `framework_adapter::FrameworkAdapter` vs real `framework_profile/adapter.rs`
    false-pass)."""
    if "::" not in sym:
        return None
    head = sym.split("::", 1)[0].strip().replace("\\", "/")
    if "/" in head and head.rsplit("/", 1)[-1].count(".") and head.endswith(
            (".rs", ".ts", ".svelte", ".py")):
        return head
    return None


def _col6_def_in_file(leaf: str, file_hint: str, sym: str, mp: str) -> List[str]:
    """File-locked col6 resolution for a produced symbol.

    The codebase_mapper writes a TRUNCATED source fragment into col2 (the
    SYMBOL column) for any DEF row preceded by a doc-comment (///) or an
    attribute (#[...]) -- e.g. col2 == "e for AntiFabricati" -- so the clean
    leaf name lives only in col6 (the SNIPPET column). `_def_hits` keys on the
    tab-bounded col2 and therefore misses these real leaves (W-1 false-negative
    this fix closes). This mirrors how `_production_caller` already matches the
    leaf in col6 and works correctly.

    A row counts as a DEFINITION of `leaf` (not a mere reference) when its col3
    file matches `file_hint` AND col6 carries a definition-shaped context:
      (a) the row tag is DEF_*/SCIP_DEF (fns / structs / enums / traits), OR
      (b) `pub static LEAF` / `static LEAF` / `pub const LEAF` / `const LEAF`
          (statics/consts are tagged TYPE_REF/CALL by the mapper, never DEF), OR
      (c) the contract names a VARIANT (`Parent::Variant`, >=2 `::` arms or a
          known *Kind/*ActionType parent) and col6 has `::LEAF(` / `Self::LEAF(`
          (the enum-variant constructor/match shape -- variants get no DEF row), OR
      (d) the contract names a FIELD (`Struct::field`) and col6 has `LEAF:`
          (the field declaration / struct-literal field-set shape).
    The `::`/`Self::` qualifier in (c) and the trailing `:` in (d) keep a bare
    textual reference from counting; the file lock keeps a wrong-path symbol out.
    """
    esc = re.escape(leaf)
    out: List[str] = []
    arms = sym.split("::")
    # heuristics for what the leaf IS, from the contract shape
    is_variant = len(arms) >= 2  # Parent::Variant or path::Parent::Variant
    is_field = len(arms) >= 2     # Struct::field shares the >=2-arm shape
    rx_const = re.compile(rf"\b(?:pub\s+)?(?:static|const)\s+{esc}\b")
    rx_variant = re.compile(rf"(?:::|\bSelf::){esc}\s*[(,=]|(?:::|\bSelf::){esc}\b\s*=>")
    rx_field = re.compile(rf"\b{esc}\s*:")
    rx_word = re.compile(rf"\b{esc}\b")
    for row in grep_map(rf"\b{esc}\b", mp):
        parts = row.rstrip("\n").split("\t")
        if len(parts) < 3:
            continue
        col3 = parts[2].rsplit(":", 1)[0].replace("\\", "/")
        if not col3.endswith(file_hint):
            continue
        col6 = parts[5] if len(parts) > 5 else ""
        if not rx_word.search(col6):
            continue
        tag = parts[0]
        if tag.startswith(("DEF_", "SCIP_DEF")):
            out.append(row)
            continue
        if rx_const.search(col6):
            out.append(row)
            continue
        if is_variant and rx_variant.search(col6):
            out.append(row)
            continue
        if is_field and rx_field.search(col6):
            out.append(row)
            continue
    return out


def check_produces(syms: List[str], mp: str,
                   project_root: str = DEFAULT_PROJECT_ROOT) -> List[Dict[str, Any]]:
    res = []
    for s in syms or []:
        if _is_path_produces(s):
            path = _resolve_produces_path(s, project_root)
            res.append({"symbol": s,
                        "status": "PASS" if path else "FAIL",
                        "hits": 1 if path else 0,
                        "kind": "file", "resolved": path})
            continue
        # Try the literal contract string first (bare contracts still match),
        # then fall back to the bare leaf, which is how elai.map keys symbols.
        rows = _def_hits(s, mp)
        leaf = _leaf_symbol(s)
        used = s
        if not any(r.startswith(("DEF_", "SCIP_DEF")) for r in rows) and leaf != s:
            leaf_rows = _def_hits(leaf, mp)
            if any(r.startswith(("DEF_", "SCIP_DEF")) for r in leaf_rows):
                rows = leaf_rows
                used = leaf
        found = any(r.startswith(("DEF_", "SCIP_DEF")) for r in rows)
        kind = "col2"
        # FALLBACK: the col2 SYMBOL column is garbled (truncated source fragment)
        # for DEF rows preceded by /// or #[...]; the clean leaf is in col6. When
        # the contract carries a source-file path, resolve the leaf in col6 of a
        # DEFINITION-shaped row in THAT file (the file lock prevents a wrong-path
        # false-pass). Mirrors the working _production_caller col6 match.
        if not found:
            fh = _file_hint(s)
            if fh:
                col6_rows = _col6_def_in_file(leaf, fh, s, mp)
                if col6_rows:
                    rows = col6_rows
                    used = f"{leaf} @col6:{fh}"
                    found = True
                    kind = "col6"
        res.append({"symbol": s, "status": "PASS" if found else "FAIL",
                    "hits": len(rows), "resolved_as": used, "via": kind})
    return res


_REF_TAGS = ("CALL_RESOLVED", "SCIP_REF", "CALL", "METHOD_CALL")


def _ref_hits_col6(leaf: str, mp: str) -> int:
    """Count CALL/REF rows whose col6 (SNIPPET) names `leaf` as a word. The
    mapper garbles col2 (the SYMBOL column) into a truncated source fragment for
    reference rows preceded by /// or #[...] exactly as it does for DEF rows, so
    a tab-bounded col2 grep undercounts real callers; the clean leaf survives in
    col6. Mirrors the working `_production_caller` col6 match. Consumes never
    affects the plan verdict (WARN floor), so this only sharpens the report."""
    esc = re.escape(leaf)
    rx = re.compile(rf"\b{esc}\b")
    n = 0
    for row in grep_map(rf"(\t{esc}\t|::{esc}\b|\b{esc}\()", mp):
        parts = row.split("\t")
        if len(parts) < 6:
            continue
        if parts[0] not in _REF_TAGS:
            continue
        if rx.search(parts[5]):
            n += 1
    return n


def check_consumes(syms: List[str], mp: str) -> List[Dict[str, Any]]:
    res = []
    for s in syms or []:
        rows = _def_hits(s, mp)
        leaf = _leaf_symbol(s)
        if not rows and leaf != s:
            rows = _def_hits(leaf, mp)
        refs = sum(1 for r in rows if r.startswith(_REF_TAGS))
        via = "col2"
        if not refs:
            # col2 garble: count references via the leaf-in-col6 path the same
            # way _production_caller resolves a real production caller.
            c6 = _ref_hits_col6(leaf, mp)
            if c6:
                refs = c6
                via = "col6"
        if refs:
            res.append({"symbol": s, "status": "PASS", "refs": refs,
                        "via": via})
        else:
            res.append({"symbol": s, "status": "WARN", "refs": 0,
                        "note": "no refs - may be macro-generated"})
    return res


def _build_adj(mp: str) -> Dict[str, Set[str]]:
    adj: Dict[str, Set[str]] = {}
    try:
        with open(mp, "r", encoding="utf-8", errors="replace") as fh:
            for ln in fh:
                if not ln.startswith("CALL_RESOLVED\t"):
                    continue
                parts = ln.rstrip("\n").split("\t")
                if len(parts) < 5:
                    continue
                m = re.search(r"<-([\w:.]+)$", parts[4])
                if m:
                    adj.setdefault(parts[1], set()).add(m.group(1))
    except OSError:
        pass
    return adj


_TEST_PATH_MARKERS = ("/tests/", "\\tests\\", "/contracts/tests/",
                      "\\contracts\\tests\\", "_test.rs", ".test.ts")


def _production_caller(symbol: str, mp: str,
                       project_root: str = DEFAULT_PROJECT_ROOT) -> Optional[str]:
    """Return the file:line of a production (non-test) CALL/REF to `symbol`.

    elai.map represents Rust intra-crate calls as CALL/SCIP_REF/METHOD_CALL rows
    (col3=file:line) rather than CALL_RESOLVED `<-caller` edges (which exist only
    for Svelte import edges). The BFS adjacency graph therefore cannot model a
    Rust call chain; this is the sound W-3 substitute: a live production caller
    exists for the symbol. Test-only callers (tests/, contracts/tests/,
    #[cfg(test)]) do NOT count (a #[cfg(test)]-only symbol is wire-dark). A row
    whose source file no longer exists on disk is skipped: SCIP indices can lag a
    file deletion, so a citation must be a file that is actually present.
    """
    leaf = _leaf_symbol(symbol)
    # The codebase_mapper records a call's symbol in col2 either bare
    # ("seed_security_rules"), method-qualified ("ProjectRuleSeeder::seed_security_rules"),
    # or garbled (truncated source fragment); the clean leaf also reliably appears
    # in the snippet (col6) as "<leaf>(". Match the leaf anywhere on the row, then
    # require a non-test CALL/REF row in an on-disk production file.
    esc = re.escape(leaf)
    rows = grep_map(rf"(\t{esc}\t|::{esc}\b|\b{esc}\()", mp)
    for row in rows:
        parts = row.split("\t")
        if len(parts) < 3:
            continue
        tag, fileline = parts[0], parts[2]
        if tag not in ("CALL", "SCIP_REF", "METHOD_CALL", "CALL_RESOLVED"):
            continue
        low = fileline.lower()
        if any(mk in low for mk in _TEST_PATH_MARKERS):
            continue
        # snippet (col6) carrying #[cfg(test)] is excluded
        snippet = parts[5].lower() if len(parts) > 5 else ""
        if "cfg(test)" in snippet:
            continue
        # guard against stale-SCIP rows citing a deleted file
        rel = fileline.rsplit(":", 1)[0]
        if not os.path.isfile(os.path.join(project_root, rel.replace("/", os.sep))):
            continue
        return fileline
    return None


def _file_referenced_in_prod(path_sym: str, mp: str,
                             project_root: str = DEFAULT_PROJECT_ROOT) -> Optional[str]:
    """A produced file artifact is wired if a production source references it
    (e.g. include_str!("../seed/security/sqli.toml") or a Svelte import). Matches
    the basename on a non-test row whose own source file exists on disk."""
    base = path_sym.replace("\\", "/").rsplit("/", 1)[-1]
    for row in grep_map(re.escape(base), mp):
        parts = row.split("\t")
        if len(parts) < 3:
            continue
        fileline = parts[2]
        low = fileline.lower()
        if any(mk in low for mk in _TEST_PATH_MARKERS):
            continue
        rel = fileline.rsplit(":", 1)[0]
        # the artifact's OWN def/file row is not a "reference"; require a
        # different source file to be the one mentioning it.
        if rel.replace("\\", "/").endswith(path_sym.replace("\\", "/")):
            continue
        if not os.path.isfile(os.path.join(project_root, rel.replace("/", os.sep))):
            continue
        return fileline
    return None


def reachable(rfrom: List[str], produces: List[str], mp: str,
              max_depth: int = 8,
              project_root: str = DEFAULT_PROJECT_ROOT) -> List[Dict[str, Any]]:
    if not rfrom or not produces:
        return []
    adj = _build_adj(mp); res = []
    # rfrom anchors in F-plan contracts carry a trailing parenthetical note;
    # reduce each to its leading symbol token (path::Symbol or path:line) so the
    # suffix match below is against the real anchor, not the prose.
    rfrom_keys = [a.split(" ", 1)[0].split("(", 1)[0].strip() for a in rfrom]
    rfrom_leaves = [_leaf_symbol(k.split(":", 1)[0]) for k in rfrom_keys]
    for tgt in produces:
        # File-path produces (.toml/.svelte) are loaded/imported, not "called".
        # The W-3 analog: the file is REFERENCED from a production source row
        # (include_str!/import) whose own file is on disk and not under tests/.
        if _is_path_produces(tgt):
            ref = _file_referenced_in_prod(tgt, mp, project_root)
            entry = {"produces_symbol": tgt, "depth_searched": 0,
                     "status": "PASS" if ref else "FAIL"}
            if ref:
                entry["via"] = f"referenced at {ref} (file artifact, load/import)"
            res.append(entry)
            continue
        ok = False; seen: Set[str] = set(); frontier = {tgt}; depth = 0
        while frontier and depth < max_depth and not ok:
            nf: Set[str] = set()
            for n in frontier:
                if n in seen:
                    continue
                seen.add(n)
                for c in adj.get(n, set()):
                    if any(c.endswith(rf) or (rf and rf in c)
                           for rf in rfrom_keys + rfrom_leaves):
                        ok = True; break
                    nf.add(c)
                if ok:
                    break
            frontier = nf; depth += 1
        entry: Dict[str, Any] = {"produces_symbol": tgt, "depth_searched": depth}
        if ok:
            entry["status"] = "PASS"
        else:
            # BFS over the Svelte-import-only adjacency cannot model Rust call
            # chains; fall back to the W-3 production-caller proof.
            pc = _production_caller(tgt, mp, project_root)
            if pc:
                entry["status"] = "PASS"
                entry["via"] = f"production caller at {pc} (BFS n/a for rust)"
            else:
                entry["status"] = "FAIL"
        res.append(entry)
    return res


def check_stub_ban(stub_ban: List[str], root: str) -> List[Dict[str, Any]]:
    res = []
    for clause in stub_ban or []:
        m = re.match(r"\s*(\S+)\s*!=\s*(.+?)\s*$", clause)
        if not m:
            res.append({"clause": clause, "status": "WARN",
                        "note": "unparsable"}); continue
        leaf = m.group(1).rsplit(".", 1)[-1]
        forbidden = _strip_q(m.group(2).strip())
        hits: List[str] = []
        for r, dirs, files in os.walk(root):
            dirs[:] = [d for d in dirs if d not in SKIP_DIRS]
            for fn in files:
                if not fn.endswith(SCAN_EXTS):
                    continue
                p = os.path.join(r, fn)
                try:
                    with open(p, "r", encoding="utf-8", errors="replace") as fh:
                        for i, ln in enumerate(fh, 1):
                            if leaf in ln and forbidden in ln:
                                hits.append(f"{p}:{i}")
                                if len(hits) >= 5:
                                    break
                except OSError:
                    continue
                if len(hits) >= 5:
                    break
            if len(hits) >= 5:
                break
        res.append({"clause": clause,
                    "status": "FAIL" if hits else "PASS", "hits": hits})
    return res


def _ssb_candidates(project_root: str) -> List[str]:
    """Real on-disk SSB locations, ELAI_SSB_DB override first."""
    if SSB_DB_ENV:
        return [SSB_DB_ENV]
    pr = project_root
    return [
        os.path.join(pr, "sessions", "ssb.db"),
        os.path.join(pr, ".elai_cc", "sessions", "ssb.db"),
        os.path.join(pr, ".elai", "ssb.db"),
    ]


def _find_ssb_db(project_root: str = DEFAULT_PROJECT_ROOT) -> Optional[str]:
    for p in _ssb_candidates(project_root):
        if p and os.path.exists(p):
            return p
    # last resort: any sessions/<entry>/ssb.db under the project root
    sroot = os.path.join(project_root, "sessions")
    if os.path.isdir(sroot):
        for ent in os.listdir(sroot):
            cand = os.path.join(sroot, ent, "ssb.db")
            if os.path.exists(cand):
                return cand
    return None


def _query_ssb(db: str, action: str, since: float) -> int:
    try:
        con = sqlite3.connect(db); cur = con.cursor()
        cur.execute("SELECT name FROM sqlite_master WHERE type='table'")
        tables = [r[0] for r in cur.fetchall()]
        if not tables:
            return 0
        table = next((c for c in ("payloads", "events", "ssb") if c in tables),
                     tables[0])
        cur.execute(f"SELECT * FROM {table} LIMIT 1")
        cols = [d[0] for d in cur.description] if cur.description else []
        ac = next((c for c in cols if any(k in c.lower()
                                          for k in ("action", "kind", "type"))), None)
        tc = next((c for c in cols if any(k in c.lower()
                                          for k in ("time", "stamp", "ts"))), None)
        if not ac:
            return 0
        if tc:
            cur.execute(f"SELECT COUNT(*) FROM {table} WHERE {ac}=? AND {tc}>=?",
                        (action, since))
        else:
            cur.execute(f"SELECT COUNT(*) FROM {table} WHERE {ac}=?", (action,))
        n = cur.fetchone()[0]; con.close()
        return int(n)
    except sqlite3.Error:
        return 0


def run_trigger(obs: Dict[str, str], timeout: int,
                project_root: str = DEFAULT_PROJECT_ROOT) -> Dict[str, Any]:
    out: Dict[str, Any] = {}; trig = obs.get("trigger")
    ea, es = obs.get("expect_artifact"), obs.get("expect_ssb")
    if not trig:
        # R1: a contract declaring an observable_effect but omitting the trigger
        # cannot be executed. The old SKIP let it pass with ZERO execution -- the
        # exact evaluator-tampering-by-omission hole. Absence of trigger = FAIL.
        return {"trigger": {"status": "FAIL",
                            "note": "no trigger field (R1: missing evidence is FAIL)"}}
    started = time.time()
    try:
        proc = subprocess.run(trig, shell=True, capture_output=True, text=True,
                              timeout=timeout, errors="replace")
        captured = (proc.stdout or "") + "\n" + (proc.stderr or "")
        out["trigger"] = {"status": "PASS" if proc.returncode == 0 else "FAIL",
                          "exit_code": proc.returncode,
                          "stdout_tail": (proc.stdout or "")[-400:],
                          "stderr_tail": (proc.stderr or "")[-400:]}
    except subprocess.TimeoutExpired:
        return {"trigger": {"status": "FAIL", "note": "timeout"}}
    # R4/R5: expect_artifact greps the trigger's OWN stdout/stderr -- agent-controlled
    # output with no causal link to wiring. It can NEVER be the load-bearing proof,
    # so it is ADVISORY ONLY and is excluded from the verdict (see main()).
    if ea:
        matched = False
        try:
            if re.search(ea, captured):
                matched = True
        except re.error:
            pass
        if not matched:
            for ln in captured.splitlines():
                if fnmatch(ln, ea):
                    matched = True; break
        out["expect_artifact"] = {"status": "PASS" if matched else "ADVISORY_MISS",
                                  "pattern": ea, "advisory": True}
    # R2/R4: expect_ssb is the LOAD-BEARING off-agent proof. db-not-found = FAIL
    # (the db not existing means the feature never ran and produced it). Zero rows =
    # FAIL. UNVERIFIED is gone.
    if es:
        db = _find_ssb_db(project_root)
        if not db:
            out["expect_ssb"] = {
                "status": "FAIL",
                "note": "expect_ssb declared but no ssb.db found -> feature never "
                        "ran (R2). Set ELAI_SSB_DB or check the project root."}
        else:
            n = _query_ssb(db, es, started)
            out["expect_ssb"] = {"status": "PASS" if n > 0 else "FAIL",
                                 "matches": n, "db": db}
    else:
        # R1: an observable_effect with a trigger but no expect_ssb has no off-agent
        # proof leg at all -- stdout cannot stand in for it.
        out["expect_ssb"] = {
            "status": "FAIL",
            "note": "no expect_ssb field (R1: off-agent proof is required)"}
    return out


def render_text(r: Dict[str, Any]) -> str:
    o = [f"WIRING_CONTRACT CHECK - {r['file']}", f"Verdict: {r['verdict']}"]
    if r.get("incomplete"):
        o += ["", "incomplete_contract (R1 - each is a FAIL):"]
        for reason in r["incomplete"]:
            o.append(f"  - {reason}")
    if r.get("static_only"):
        o += ["", f"static_only: {r['static_only']}"]
    o += ["", "produces_symbols:"]
    for x in r.get("produces", []):
        o.append(f"  {x['symbol']} | {x['status']} | hits={x.get('hits', 0)}")
    o += ["", "consumes_symbols:"]
    for x in r.get("consumes", []):
        o.append(f"  {x['symbol']} | {x['status']} | refs={x.get('refs', 0)} "
                 f"{x.get('note', '')}")
    o += ["", "reachable_from:"]
    for x in r.get("reachable", []):
        o.append(f"  {x['produces_symbol']} | {x['status']} | "
                 f"depth={x.get('depth_searched', 0)}")
    o += ["", "stub_ban:"]
    for x in r.get("stub_ban", []):
        o.append(f"  {x['clause']} | {x['status']} | "
                 f"hits={','.join(x.get('hits', []))}")
    if r.get("trigger"):
        o += ["", "observable_effect:"]
        for k, v in r["trigger"].items():
            o.append(f"  {k}: {v}")
    return "\n".join(o)


def main() -> int:
    ap = argparse.ArgumentParser(
        description="Verify a B-N WIRING_CONTRACT block (fail-closed)")
    ap.add_argument("--bn-file", required=True)
    ap.add_argument("--map", default=DEFAULT_MAP)
    ap.add_argument("--project-root", default=DEFAULT_PROJECT_ROOT)
    ap.add_argument("--json", action="store_true")
    ap.add_argument("--run-trigger", action="store_true",
                    help="deprecated no-op: the trigger always runs (R3)")
    ap.add_argument("--static-only", action="store_true",
                    help="debug: skip the runtime trigger; a runtime contract can "
                         "then never PASS (it is reported INCOMPLETE)")
    ap.add_argument("--allow-missing-contract", action="store_true",
                    help="debug: a file with no contract block returns SKIP not FAIL")
    ap.add_argument("--timeout", type=int, default=60)
    args = ap.parse_args()
    block = extract_block(args.bn_file)
    if block is None:
        # R1 / ArchUnit failOnEmptyShould: a feature gate pointed at a file with no
        # contract has zero evidence -> FAIL by default. --allow-missing-contract is
        # the explicit debug opt-out for incidental (non-feature) scans.
        if args.allow_missing_contract:
            if args.json:
                print(json.dumps({"file": args.bn_file, "verdict": "SKIP",
                                  "note": "no contract block (allowed)"}))
            else:
                print(f"SKIP: WIRING_CONTRACT block not found in {args.bn_file}")
            return 2
        msg = ("no WIRING_CONTRACT block (a feature gate requires one; pass "
               "--allow-missing-contract only for non-feature files)")
        if args.json:
            print(json.dumps({"file": args.bn_file, "verdict": "FAIL", "note": msg}))
        else:
            print(f"FAIL: {msg} in {args.bn_file}")
        return 1
    parsed = parse_loose_yaml(block)
    p = parsed.get("produces_symbols") or []
    c = parsed.get("consumes_symbols") or []
    rf = parsed.get("reachable_from") or []
    sb = parsed.get("stub_ban") or []
    obs = parsed.get("observable_effect") or {}
    p = p if isinstance(p, list) else []
    c = c if isinstance(c, list) else []
    rf = rf if isinstance(rf, list) else []
    sb = sb if isinstance(sb, list) else []
    obs = obs if isinstance(obs, dict) else {}

    # R1: every load-bearing field is required. Absence of any = FAIL.
    incomplete: List[str] = []
    if not p:
        incomplete.append("produces_symbols missing/empty")
    if not rf:
        incomplete.append("reachable_from missing/empty")
    if not obs:
        incomplete.append("observable_effect missing/empty")
    else:
        if not obs.get("trigger"):
            incomplete.append("observable_effect.trigger missing")
        if not obs.get("expect_ssb"):
            incomplete.append(
                "observable_effect.expect_ssb missing (off-agent proof required)")
    # fail-on-empty: a contract block present but with nothing load-bearing in it.
    if not p and not rf and not obs and not sb and not c:
        incomplete = ["contract block present but empty (R1 fail-on-empty)"]

    rp = check_produces(p, args.map, args.project_root)
    rc = check_consumes(c, args.map)
    rr = reachable(rf, p, args.map, project_root=args.project_root)
    rs = check_stub_ban(sb, args.project_root)

    trig: Dict[str, Any] = {}
    static_note = ""
    if obs:
        if args.static_only:
            # R3: the only opt-out can never manufacture a PASS for a runtime contract.
            static_note = ("runtime evidence not verified (--static-only); a runtime "
                           "contract can never PASS in static-only mode")
        else:
            trig = run_trigger(obs, args.timeout, args.project_root)

    # expect_artifact is advisory and excluded from the verdict; trigger + expect_ssb
    # feed it.
    trigger_failed = any(
        isinstance(v, dict) and not v.get("advisory") and v.get("status") == "FAIL"
        for v in trig.values())
    failed = (bool(incomplete)
              or bool(static_note)
              or any(x["status"] == "FAIL" for x in rp)
              or any(x["status"] == "FAIL" for x in rr)
              or any(x["status"] == "FAIL" for x in rs)
              or trigger_failed)
    verdict = "FAIL" if failed else "PASS"
    out = {"file": args.bn_file, "verdict": verdict, "produces": rp,
           "consumes": rc, "reachable": rr, "stub_ban": rs, "trigger": trig,
           "incomplete": incomplete}
    if static_note:
        out["static_only"] = static_note
    print(json.dumps(out, indent=2) if args.json else render_text(out))
    return 0 if verdict == "PASS" else 1


if __name__ == "__main__":
    sys.exit(main())
