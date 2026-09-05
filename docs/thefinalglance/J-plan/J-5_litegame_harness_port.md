# J-5: litegame harness port — verified half-day port of the matured harness pieces

STATUS: PLANNED (WAVE 5; PORT per JC-6 — compressed form; depends J-3 + J-4 shipped and stable
here, + J-7's scanner resolution; effort S-M, ~half a day)

- Track J | lever REACH | grounding MODERATE (target root unverified since the F:-move) | nature
  PORT | WAVE 5 | depends J-3, J-4, J-7
- Premise: D:\.liteGAME_workflow runs an older copy of this harness. Port the pieces that matured
  here — the local-rag MCP registration, the J-3 backup hook + undo tool, the J-4 ledger, and the
  F-1-style compound PreToolUse shape — AFTER verifying the root's compat state. The redesign doc
  itself scopes this as "a half-day port, not a plan" (JC-6); J-5 exists so the port has a
  checklist, a verification gate, and a done-gate instead of being an ad-hoc copy.

## Steal/build (the checklist, in order)
0. COMPAT VERIFY FIRST (blocking): the 7 liteGAME compat files never re-checked after the
   D:->F: ELAI move (memory: project_litegame_orchestrate_compat; only compat/ELAI.md was
   corrected 2026-06-05). Enumerate, read, fix stale F:/D: references. Findings land in THAT
   root's reasoning log. Any deeper rot found -> stop and report; do not port onto a broken base.
1. local-rag MCP registration for the liteGAME root: safe multi-root since the 2026-06-05
   paths.py resolver unification (per-session by CLAUDE_CODE_SESSION_ID, root by outermost
   workflow/ marker, PID-liveness — memory: project_fsm_resolver_unified). Register, then prove
   isolation (step's negative arm below).
2. J-3 port: copy+adapt pre_edit_backup.py + undo.py (paths only; logic byte-identical where
   possible); register in THAT root's settings; selftest there.
3. J-4 port: copy+adapt handoff_ledger.py; THAT root's handoff rule gains the same one line.
4. Compound PreToolUse shape: where liteGAME has stacked single-purpose hooks, consolidate new
   additions per the F-1 doctrine (one compound fail-closed evaluator with a measured FP bound,
   not N stacked hooks). EXISTING liteGAME hooks are not rewritten this pass (scope discipline) —
   the shape applies to what J-5 ADDS.
5. Skill-import gate: any skill ported into that root passes skillspector --no-llm first (J-7's
   resolved scanner; SRC-2) and logs to that root's skill-imports.log per skill-import-policy.
   EXCLUDED from this port (recorded so it is not re-litigated): the orchestrate skill, the full
   rule corpus, canvas — per-root authoring decisions belong to that root (J-0 DECLINES).

## Existing ground truth
- the steal-repo clones already live in the liteGAME root (SRC-2: workflow/external-skills/
  repos/) — the root is active and reachable.
- compat/ELAI.md corrected 2026-06-05; the OTHER 7 compat files UNVERIFIED (the named risk).
- paths.py resolver SHIPPED + 53/53 tests (memory) — the multi-root safety basis.
- J-3/J-4 artifacts exist in THIS root by WAVE-5 time (their own JD-1 gates passed).

## REVISED scope (W-10)
SHIP NOW: steps 0-5 above, copy+adapt, per-root config. DEFER: any liteGAME-specific hook
consolidation beyond what J-5 adds; any shared-code scheme between roots (explicitly banned —
copy+adapt, never shared mutable config; the D:-ghost incident class is the reason).

## Integration
D:\.liteGAME_workflow\.claude\ (tools, settings) + its rag/hooks seam + its workflow/ tree.
Propagation precedent: docs/strategy/WIRING-FIX-PROPAGATION.md (apps/liteGAME/REGRET pending
targets). frontend_pair: N/A.

## WIRING_CONTRACT
produces_symbols:
  - liteGAME copies of pre_edit_backup.py / undo.py / handoff_ledger.py (path-adapted)
  - liteGAME local-rag MCP registration + that root's settings hook entries
  - that root's compat-verify findings note (step 0 artifact)
consumes_symbols:
  - paths.py multi-root resolver (shipped — relied on, untouched)
  - J-3/J-4 sources (this root; copy+adapt)
  - skillspector --no-llm (J-7) for any skill import
reachable_from:
  - a LIVE liteGAME session's Edit/Write (the ported hook fires there)  [JD-1, that root]
stub_ban:
  - NO port before step 0 completes (compat-verify is blocking)
  - NO shared mutable config or symlinked tooling between roots
  - NO orchestrate/rule-corpus/canvas port (out of scope, recorded)
  - NO summary-only port of anything (W-3 — sources are read in this root first)
frontend_pair: N/A — harness tooling; justified.
observable_effect:
  trigger: "a real liteGAME session performs an Edit; then that root's undo.py list +
    handoff_ledger.py run; then mcp_fsm_get_state from that session"
  expect_artifact: "a backup lands under THAT root's workflow/sessions/{id}/backups/ + that
    root's ledger derives the row + fsm state resolves to the liteGAME root. NEGATIVE: a
    deliberately wrong-root probe (running this root's session id against that root's resolver)
    is REFUSED by the resolver (no F:-ghost recurrence); step-0 findings file exists even when
    empty-of-issues (the verify step provably ran, R1 analog)."

## Grounding
SRC-1 N-5; memories project_litegame_orchestrate_compat + project_fsm_resolver_unified +
project_wiring_gate_failclosed_shipped (propagation manifest). JC-6 (port-not-plan). J-0 DECLINES
(scope edges).

## COMPOSES-WITH
J-3/J-4 (the ported artifacts), J-7 (the scanner gate), F-1 (compound-hook doctrine), the
paths.py resolver. Restates none.

## Open questions / risks
- the 7 compat files may surface deeper drift than path strings — step 0's stop-and-report rule
  covers it (honest, never port-over-rot).
- liteGAME's hook inventory may differ structurally; J-5 ADDS alongside, consolidation is that
  root's own future call.

## Next
The harness floor (backups, ledger, scanner-gated imports, resolver-safe MCP) becomes
two-root-standard; apps/REGRET remain on the propagation manifest for their own future pass.

End of J-5.
