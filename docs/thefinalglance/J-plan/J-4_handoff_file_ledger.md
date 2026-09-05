# J-4: handoff file ledger — FILES TOUCHED derived from recorded state, never recollection

STATUS: PLANNED (WAVE 1; BUILD, harness-side only; effort S)

Objective: the handoff protocol's FILES TOUCHED section is authored from the narrating LLM's
recollection — exactly the failure mode G-5's DSR principle and H-51's journal-match rule exist to
kill on the elai side. The data already exists: PostToolUse change-recording to per-session state
is live (HG-4, proven by the wire-remediation incident that had to edit state.json changes[]).
J-4 ships .claude/tools/handoff_ledger.py — derive every file edited this session (path, edit
count, last-good backup seq from J-3 when present, audit verdict if recorded) — plus ONE line in
the handoff-protocol template requiring the machine block whenever the tool is available (JD-2).
Hard boundary: the one-file-per-problem handoff rule and the entry template's other sections are
untouched; this is one tool + one template line, not a protocol rewrite.

Estimated effort: ~0.3 session (~2-3 h). Pure stdlib python, zero network.
Pre-requisites: none (J-3's last-good column is optional — the ledger ships without it and gains
the column when J-3 lands).
Parallel-safe with: everything (coordinate session-state schema reading with J-3, not files).
Source: SRC-1 N-4. Doctrine: J-0_INDEX.md (JD-1/JD-2 bind).

## Context
The user's ask: "better handoff that shows every file edited". H-51 enforces the elai-side analog
(digest DECISIONS/FILES must match journal FileWrite rows — off-agent, not recollection); the
harness mirrors it for near-zero cost because the hooks already record changes. The ledger is also
the cheap instrument that makes every FUTURE session's handoff partially machine-verified: a
reviewer can diff the narrated entry against the derived block.

## Existing state (why this is BUILD)
- HG-1: no handoff_ledger.py exists.
- HG-4: session state records changes (state.json changes[]) + hook-executions.log rows exist
  per-session; exact schemas PLAN-asserted — read both shapes defensively.
- .claude/rules/handoff-protocol.md: live, entry template carries FILES TOUCHED as prose;
  one-file-per-problem + head-only-append rules in force (untouched by J-4).
- J-3 (when shipped): backups/{...}@{seq} tree + last-good marks — the optional column source.

## Reads (mandatory before coding)
1. _grounding/J-HARNESS-GROUNDING-MAP.md (HG-4, JC-2).
2. .claude/rules/handoff-protocol.md (the FILE TEMPLATE + ENTRY SECTIONS — the ONE line lands in
   FILES TOUCHED's definition).
3. workflow/sessions/<two recent sessions>/ state.json + hook-executions.log (real shapes, both).
4. SRC-1 N-4 (the ask + the G-5 derivation principle).

## Tasks (WAVE 1)
1. handoff_ledger.py (~150 LOC stdlib): `--session <sid>` (default: newest live session) ->
   derive per-file rows {path, edit_count, first_seq, last_seq, last_good_seq?, audit_verdict?}
   from state.json changes[] AND hook-executions.log (union, dedup); emit a paste-ready block:
   one line per file, stable ordering (path-sorted), header naming the derivation sources +
   timestamp. `--json` for tooling.
2. Honest-degradation arm: missing/unreadable session state -> print exactly
   "LEDGER UNAVAILABLE (no recorded session state at <path>)" and exit 1 — never an empty silent
   section (R1 analog; the dead evidence path screams, R2).
3. Template amendment (ONE line in handoff-protocol.md's ENTRY SECTIONS, FILES TOUCHED): when
   .claude/tools/handoff_ledger.py exists, the FILES TOUCHED body MUST be (or include verbatim)
   its output block; hand-tallied-only entries are a JD-2 violation at review.
4. Self-test mode (`--selftest`, ~50 LOC): temp fixture session dir with synthetic state ->
   asserts row derivation, union/dedup, degradation arm.
5. Decision doc: workflow/reasoning/J-4-decisions.md (schema findings from the two real sessions,
   the union rule, the J-3 column contract).

## WIRING_CONTRACT
produces_symbols:
  - .claude/tools/handoff_ledger.py (the derivation tool + selftest)
  - the handoff-protocol.md FILES TOUCHED machine-block requirement (one line)
consumes_symbols:
  - workflow/sessions/{id}/state.json changes[] + hook-executions.log (HG-4; read-only)
  - J-3 backups tree + last-good marks (OPTIONAL column; absent -> column omitted, stated)
  - .claude/rules/handoff-protocol.md (amended one line; one-file rule untouched)
reachable_from:
  - every handoff authoring per the protocol (the main LLM runs the tool when writing/appending
    a handoff)  [HARNESS PRODUCTION, JD-1]
stub_ban:
  - NO recollection-sourced rows (every row traceable to a recorded state line, JD-2)
  - NO silent empty ledger (degradation prints + exits nonzero)
  - NO non-stdlib import; NO schema guess hard-coded (defensive dual-shape read)
frontend_pair:
  N/A — harness tooling; justified.
observable_effect:
  trigger: "python .claude/tools/handoff_ledger.py --session <current> during a real handoff
    authoring"
  expect_artifact: "a derived block whose rows match the session's hook log (spot-check
    provable); the next real handoff entry carries the block verbatim. NEGATIVE: pointing the
    tool at a session with no recorded state prints LEDGER UNAVAILABLE + exit 1; a handoff entry
    with hand-tallied FILES TOUCHED while the tool exists is flagged at review (JD-2)."

## DETERMINISTIC_CHECK
```bash
python .claude/tools/handoff_ledger.py --selftest                      # exit 0, all asserts pass
python .claude/tools/handoff_ledger.py --session <sid> --json          # rows match hook log spot-check
rg "handoff_ledger" .claude/rules/handoff-protocol.md                  # the template line landed
rg -c "import (blake3|zstd)" .claude/tools/handoff_ledger.py           # MUST be 0
test -f workflow/reasoning/J-4-decisions.md
```

## ACCEPTANCE
- [ ] rows derived from recorded state only (JD-2); union/dedup across both shapes.
- [ ] degradation arm loud + nonzero (R1/R2 analogs).
- [ ] one-line template amendment; the one-file handoff rule + head-only-append untouched.
- [ ] stdlib-only; selftest green.
- [ ] the FIRST real handoff after shipping carries the machine block (JD-1 evidence).

## Tests to add
- the in-tool --selftest (counts as the contract test for a harness tool): (a) derivation,
  (b) union/dedup, (c) degradation, (d) stable ordering, (e) J-3-column omission when backups
  absent.

## Files touched
- CREATE: .claude/tools/handoff_ledger.py (~200 LOC incl selftest)
- MODIFY: .claude/rules/handoff-protocol.md (ONE line)
- CREATE: workflow/reasoning/J-4-decisions.md

## Citations / Grounding
- SRC-1 N-4; HG-4 (the recorded-state proof); G-5 DSR principle + H-51 journal-match (the
  doctrine sources, cited not restated); JC-2.

## Open questions / risks
- state schema variance across roots/sessions: dual-shape defensive read + the decisions doc
  records the shapes found; a third shape appearing later extends the reader, never silently
  drops rows.
- sessions predating the hooks: ledger says UNAVAILABLE honestly.

## Next
Every subsequent J wave (and every session, period) hands off with machine-derived file evidence.
J-5 ports it; J-3's last-good column slots in when its backups land.

End of J-4.
