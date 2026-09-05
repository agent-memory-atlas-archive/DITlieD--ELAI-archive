# J-3: harness undo + per-session file backups — every LLM edit in THIS workflow reversible

STATUS: PLANNED (WAVE 2; BUILD, harness-side only; non-blocking by construction — JD-3)

Objective: H-49/I-52/E-25 give ELAI undo; NOTHING protects the harness that builds it — the
.claude/ + workflow/ tooling root where the main LLM edits rules, tools, plans, and memory-bank
files daily. git is no substitute: this root gitignores everything except .elai_cc, so an LLM
mis-edit to an uncommitted plan/rule/tool is simply GONE (HG-1: no undo tooling exists). J-3 ships
the claude-code-workflow half of U3: a NEW PreToolUse hook that copies the pre-image of every
Edit/Write/NotebookEdit target into the session dir BEFORE the write lands, plus
.claude/tools/undo.py to list/diff/restore/prune — with H-49's RV-1 semantics (a restore NEVER
clobbers manual edits; it refuses and shows the diff). Hard boundaries: the existing registered
hooks are NEVER edited (JD-4/R8 — a new hook file is added and registered as a user-reviewed
settings change); the tool is stdlib-only (HG-1 house convention) which forces two documented
deviations from the source doc: gzip not zstd (JC-1) and hashlib.blake2b not blake3 (JC-2, with a
stored algorithm prefix so a later py-blake3 migration is additive).

Estimated effort: ~0.5-1 session (~4-6 h). Tier: pure in-process python, zero network, zero deps.
Pre-requisites: none (the hook seam HG-3 + the session-state recording HG-4 already exist).
Parallel-safe with: everything except J-4 (J-4 reads the same session-state shapes; coordinate
the schema reading, not the files).
Source: SRC-1 N-3; prior-art read: headroom/cache/compression_store.py (SRC-2, reference only).
Doctrine: J-0_INDEX.md (JD-1..JD-4 bind hardest here).

## Context
The ask, verbatim from the redesign doc: "every change the llm makes to a file, that file is then
made backup of in the session... revert at any moment" — for BOTH systems. ELAI's half is A-1
(H-35 -> H-49 -> I-52, existing files). This is the harness half. The mechanism is deliberately
boring: content-addressed pre-image copies + a divergence-refusing restore. The one piece of
intelligence is last-good marking: the session state the PostToolUse hook already records (HG-4)
knows which edits preceded an audit PASS; `undo.py prune` keeps last-good + newest and drops the
middle — simple last-PASS-wins, deliberately NOT the D-5 bandit machinery (SRC-1's own steer).

## Existing state (why this is BUILD)
- HG-1: 26 tools in .claude/tools/, no undo/backup tooling.
- HG-3: live hook seam = C:/Users/user/.claude/rag/hooks/ (pre_tool_check.py + siblings),
  registered via .claude/settings.local.json (re-read at edit), logging to per-session
  workflow/sessions/{id}/hook-executions.log.
- HG-4: PostToolUse change-recording to session state EXISTS (state.json changes[] — proven by
  the wire-remediation incident). The exact schema is PLAN-asserted; read defensively.
- HG-6: the stale-F:-hook bug (FSM gate no-op for F: paths) is ADJACENT and deliberately unfixed —
  named open question below; J-3 does not touch pre_tool_check.py.
- H-49 (elai side): RV-1 divergence guard + undo-tree semantics — the semantics source J-3 mirrors
  WITHOUT porting the CAS/undo-tree machinery (linear per-file history is enough for the harness).

## Reads (mandatory before coding)
1. _grounding/J-HARNESS-GROUNDING-MAP.md (HG-1/HG-3/HG-4/HG-6, JC-1/JC-2).
2. SRC-1 N-3 (the ask + the last-good design) + H-49's RV-1 definition (semantics only).
3. .claude/settings.local.json (the live hook registration shape) + one existing hook file as the
   pattern (structure, logging, exit codes) — read, never edit.
4. workflow/sessions/<a recent session>/ (state.json + hook-executions.log actual shapes).
5. headroom/cache/compression_store.py (SRC-2 clone; reversible-store prior-art; idea-not-dep).

## Tasks (WAVE 2)
1. Backup hook (~120 LOC python, stdlib): NEW file at the hook seam (name it distinctly, e.g.
   pre_edit_backup.py). On PreToolUse for Edit/Write/NotebookEdit: read target pre-image; hash
   (blake2b keyed, algorithm-prefixed per JC-2); if hash unseen this session, write
   workflow/sessions/{id}/backups/{hash[:12]}/{relpath-encoded}@{seq} (gzip above ~64KB, JC-1);
   append a backup line to hook-executions.log; NEVER block the tool call (JD-3 — observe-only;
   on any internal error, log + allow).
2. Registration: a settings.local.json entry ADDING the new hook alongside existing ones —
   presented as a reviewable diff for the user (JD-4: registration is a user-reviewed change;
   the plan ships the hook file + the exact JSON line to add).
3. undo.py (~200 LOC, stdlib): subcommands list (per-file timeline with seq/size/hash/last-good
   mark), diff (pre-image vs on-disk, unified), restore (by file or seq; RV-1: if on-disk hash !=
   recorded post-image hash -> REFUSE + print diff + require --force-after-review), prune
   (keep last-good + newest per file; dry-run default).
4. Last-good marking (~40 LOC): derive from session state (HG-4): the newest backup seq preceding
   a recorded audit-PASS event is tagged last-good in list output. Absent audit records -> no
   last-good claimed (honest, R1 analog).
5. Smoke + negative arms: a scripted session-shaped fixture (temp session dir) proving: backup
   lands BEFORE the edit (seq/mtime ordering), restore round-trips bytes (hash match), divergence
   arm refuses, unregistered-hook arm produces zero backups AND undo.py says so loudly (R2).
6. Decision doc: workflow/reasoning/J-3-decisions.md (hash/compression deviations JC-1/JC-2, the
   size threshold, the prune policy, the HG-6 open question verbatim).

## WIRING_CONTRACT
produces_symbols:
  - rag/hooks/pre_edit_backup.py (the new PreToolUse backup hook; NEW file)
  - .claude/tools/undo.py (list/diff/restore/prune)
  - workflow/sessions/{id}/backups/ (the artifact tree)
consumes_symbols:
  - the hook registration seam (settings.local.json; user-reviewed addition)
  - session state / hook-executions.log (HG-4 — read-only derivation)
  - H-49 RV-1 semantics (mirrored; machinery NOT ported — GD-1)
reachable_from:
  - every live-session Edit/Write/NotebookEdit once registered  [HARNESS PRODUCTION, JD-1]
stub_ban:
  - NO edit to pre_tool_check.py or any existing registered hook (JD-4/R8)
  - NO blocking verdict from the backup hook (JD-3 — observe-only; errors log+allow)
  - NO silent restore over a diverged file (RV-1; --force requires the shown diff)
  - NO non-stdlib import (JC-1/JC-2 deviations documented in-file)
  - NO recollection-derived last-good (state-derived only, JD-2)
frontend_pair:
  N/A — harness tooling with a CLI surface; justified (the JD-1 artifact is the backup tree +
  log rows).
observable_effect:
  trigger: "a real harness session performs an Edit on a workflow file; then `python
    .claude/tools/undo.py list` / `restore` / the divergence arm"
  expect_artifact: "the pre-image file exists under the session backups dir with a
    hook-executions.log row BEFORE the edit's own record (JD-1); restore returns byte-identical
    pre-image (hash match); restore onto a hand-modified file REFUSES with a diff. NEGATIVE: with
    the hook unregistered, zero backups land and undo.py reports 'no backup hook active for this
    session' (the dead path screams, R2)."

## DETERMINISTIC_CHECK
```bash
python .claude/tools/undo.py list --session <sid>                      # >=1 row after a fixture edit
python .claude/tools/undo.py restore --file <f> --dry-run              # prints hash-verified plan
rg -c "import (zstd|blake3)" .claude/tools/undo.py rag/hooks/pre_edit_backup.py   # MUST be 0 (JC-1/JC-2)
rg "blake2b" .claude/tools/undo.py                                     # algorithm-prefixed hashing present
test -f workflow/reasoning/J-3-decisions.md
```

## ACCEPTANCE
- [ ] backup precedes the write (ordering provable from seq + log).
- [ ] restore round-trip byte-exact; divergence refusal proven; prune keeps last-good + newest.
- [ ] hook is observe-only (JD-3); a hook crash cannot block an edit (error path tested).
- [ ] no existing hook edited (JD-4); registration shipped as a reviewable settings diff.
- [ ] stdlib-only; JC-1/JC-2 deviations documented in the decisions doc.
- [ ] HG-6 (stale-F:-hook) recorded as the named user-deliberate open question — untouched.
- [ ] friction observed across >=3 real sessions before any future blocking behavior is even
      proposed (JD-3/GD-4).

## Tests to add
- a stdlib self-test mode in undo.py (`undo.py selftest` over a temp session fixture, ~80 LOC):
  (a) backup-before-edit ordering; (b) restore hash match; (c) divergence refusal; (d) prune
  policy; (e) corrupt/truncated backup -> typed failure, never a silent partial restore;
  (f) unregistered-hook negative arm.

## Files touched
- CREATE: rag/hooks/pre_edit_backup.py (~120 LOC)
- CREATE: .claude/tools/undo.py (~280 LOC incl selftest)
- MODIFY: .claude/settings.local.json (ONE hook registration line, user-reviewed)
- CREATE: workflow/reasoning/J-3-decisions.md

## Citations / Grounding
- SRC-1 N-3 (the ask, the last-good design, the not-D-5 steer). HG-1/HG-3/HG-4/HG-6.
- JC-1 (gzip), JC-2 (blake2b + migration path). H-49 RV-1 (semantics source).
- headroom compression_store.py (SRC-2; prior-art read only).

## Open questions / risks
- HG-6 stale-F:-hook fix: USER-DELIBERATE, out of J-3's scope; until fixed, the FSM gate's F:
  no-op means J-3's protection matters MORE (it is the only pre-write net on this root).
- backup volume on heavy sessions: bounded by dedup + threshold-gzip + prune; observe across 3
  sessions before tuning (JD-3).
- state.json schema variance (HG-4 PLAN-asserted): the last-good derivation reads defensively and
  degrades to "no last-good" honestly.

## Next
The harness gets the same floor it built for ELAI. J-4's ledger cross-references last-good seqs;
J-5 ports both to liteGAME; every later wave's edits are recoverable from day one.

End of J-3.
