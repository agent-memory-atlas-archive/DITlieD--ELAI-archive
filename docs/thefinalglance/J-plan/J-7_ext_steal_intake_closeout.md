# J-7: ext-steal intake closeout — discharge the 2026-06-10 steal doc's pending actions as one gated unit

STATUS: PLANNED (WAVE 2; EXTENSION, harness-side; effort S, <2h; SOURCE = the ext-repo steal doc
[SRC-2], NOT a Section B item — the +1 the J index declares; the whole unit is pending the user's
yes, which this file is the approval surface for)

- Track J | lever AUDITABLE | grounding STRONG (skillspector interface map is source-reviewed) |
  nature EXTENSION | WAVE 2
- Premise: the ext-repo steal doc closed its analysis with three concrete "pending your yes"
  actions and ~9 pointer-line insertions. Left ownerless, they rot. J-7 bundles them as one
  session with a done-gate, fully discharging BOTH 2026-06-10 reasoning docs through the J plan.

## Steal/build (the three actions + the pointer pass)
A. SKILLSPECTOR = the resolved scanner (highest value):
   - smoke from the clone: `skillspector scan <one of our own .claude/skills/*> --no-llm -f json`
     (STATIC ONLY per the interface map: deterministic, no API key, no egress except optional
     OSV.dev with silent offline fallback).
   - parse findings -> ONE TSV row in workflow/skill-imports.log per the existing schema
     (ts, skill_name, skill_dir, scan_tool, findings_count, scan_hash, action). scan_hash =
     hash of scanner output; JC-2 governs (blake2b stdlib now, algorithm-prefixed; blake3 on dep
     approval — the policy's "blake3" wording gains a parenthetical).
   - amend .claude/rules/skill-import-policy.md item 1: scanner RESOLVED = skillspector --no-llm
     (NVIDIA-org GitHub = the verifiable channel mcp-scan lacks; mcp-scan demoted to optional
     second leg if its dist-channel ever verifies). RECORD the binding caveats from the interface
     map: static precision MODERATE -> findings are the reviewable artifact, the 0-100 score is
     advisory only; score SATURATES (log findings_count + per-finding severities, never just the
     score); executable detection is extension-only; the mcp_rug_pull analyzer is a STUB (RP1-RP3
     unimplemented) -> G-4's re-pin gate REMAINS the only MCP rug-pull coverage (named
     non-coverage, GD-5).
B. IMPECCABLE = deterministic design lint intake:
   - anti-slopsquat first: `npm view impeccable version dist-tags time` (recent publish date
     required per the protocol).
   - trial: `npx impeccable detect --fast --json` over dashboard-v2 src; record findings count +
     runtime in the decisions doc.
   - add ONE line to the dashboard-smoker checklist (the dashboard-handoff skill's check set):
     run impeccable --fast on changed .svelte/.css. CLI-only use = no SKILL.md import = no
     skill-import scan triggers. HOLD stands on importing the 23-command impeccable skill itself
     (>3-verb scope-creep flag; if ever wanted: whole-dir scan + log row + approval first).
C. POINTER PASS (~9 one-line GD-1 insertions, exactly as the steal doc lists):
   "external prior-art: <repo path> -> <files>" into F-1 (skillspector pattern sources for the
   Tier-2 malicious-skill axis), B-29/B-30/E-22 (codegraph), I-12/I-14/I-40 (headroom),
   H-12/H-14 (Understand-Anything concepts + impeccable live/*.mjs). One line each, no scope
   changes to those plans.
   BINDING for all of A-C: the STEAL METHOD — any later reimplementation reads the named source
   files from the clones; a summary-only port is a W-3 violation. Licenses (Apache-2.0/MIT)
   noted with attribution in any adapted-file header.

## Existing ground truth
- skill-import-policy.md item 1's scanner slot is THE documented unresolved gap (mcp-scan
  dist-channel UNVERIFIED — the policy's own open question + G-4's carried correction).
- workflow/papers/skillspector-interface.md exists (paper-ingestor, source-reviewed) — the
  binding scoping source for action A.
- clones on disk at D:\.liteGAME_workflow\workflow\external-skills\repos\ (SRC-2, verified
  2026-06-10).
- declines already recorded (agentmemory, Agent-Reach, gajae wholesale) — J-7 does NOT reopen
  them (J-0 DECLINES section).

## REVISED scope (W-10)
SHIP NOW: A + B + C as above. DEFER: the F-1 Tier-2 Rust reimplementation of skillspector's
pattern inventory (a real build session against the listed sources — belongs to the F-1 follow-up
its pointer line marks); any impeccable rule-engine native port (same).

## Integration
workflow/skill-imports.log (+1..2 rows); .claude/rules/skill-import-policy.md (item-1 amendment);
the dashboard-handoff skill checklist (one line); 9 plan files (one line each). frontend_pair: N/A.

## WIRING_CONTRACT
produces_symbols:
  - the skill-imports.log row(s) with findings_count + per-finding severities
  - the amended skill-import-policy.md item 1 (scanner resolved + stub caveat)
  - the smoker checklist line + the 9 pointer lines
consumes_symbols:
  - workflow/papers/skillspector-interface.md (the binding scoping corrections)
  - the skillspector + impeccable clones (run-from-clone; no install into this root)
  - .claude/rules/skill-import-policy.md schema (the TSV shape — extended in meaning, not format)
reachable_from:
  - the skill-import gate itself: every FUTURE external-skill import now runs the resolved
    scanner (the policy is the production path)  [HARNESS PRODUCTION, JD-1]
stub_ban:
  - NO auto-trust of the 0-100 score (findings are the artifact; score advisory)
  - NO claim of MCP rug-pull coverage (the stub non-coverage is named; G-4 remains the defense)
  - NO impeccable skill import (HOLD stands); NO new dep into this root (run from clone / npx)
  - NO summary-only future port (the source-file lists are the contract)
frontend_pair: N/A — harness policy/tooling; justified.
observable_effect:
  trigger: "run action A's scan against one of our own skills + action B's npm-view + trial;
    insert C's lines"
  expect_artifact: "a skill-imports.log row exists with findings_count + severities + action
    (ALLOWED or NEEDS_REVIEW); the policy file names skillspector as resolved WITH the rug-pull
    caveat; `npm view` output recorded in the decisions doc; each of the 9 pointer lines
    grep-present in its plan file. NEGATIVE: scanning a seeded malicious-pattern fixture skill
    yields HIGH/CRITICAL findings -> action=BLOCKED row and the import is refused (the gate
    bites, R2)."

## DETERMINISTIC_CHECK
```bash
rg "skillspector" .claude/rules/skill-import-policy.md                  # scanner resolved + caveat present
rg "rug.?pull" .claude/rules/skill-import-policy.md                     # the non-coverage is named
tail -5 workflow/skill-imports.log                                      # the new row(s), findings_count populated
rg "external prior-art" docs/thefinalglance/ -l                         # >=9 plan files carry pointer lines
test -f workflow/reasoning/J-7-decisions.md
```

## ACCEPTANCE
- [ ] scan run --no-llm from the clone; row logged per schema; score treated as advisory.
- [ ] policy amendment carries BOTH the resolution and the stub non-coverage (GD-5).
- [ ] impeccable npm-verified before any npx run; smoker line added; skill import still HELD.
- [ ] all 9 pointer lines inserted, one line each, no scope edits to those plans (GD-1).
- [ ] negative arm (seeded malicious fixture -> BLOCKED) demonstrated.
- [ ] no new dependency installed into this root.

## Files touched
- MODIFY: .claude/rules/skill-import-policy.md (item 1)
- MODIFY: workflow/skill-imports.log (+rows)
- MODIFY: the dashboard-handoff skill checklist (one line)
- MODIFY: 9 plan files (one pointer line each: F-1, B-29, B-30, E-22, I-12, I-14, I-40, H-12, H-14)
- CREATE: workflow/reasoning/J-7-decisions.md

## Citations / Grounding
SRC-2 (rev 2, the action list + binding corrections) + workflow/papers/skillspector-interface.md;
JC-2 (hashing); J-0 DECLINES (what stays declined). Licenses: skillspector Apache-2.0, impeccable
Apache-2.0 — attribution preserved on any future adaptation.

## Open questions / risks
- skillspector platform needs (Python 3.12+, yara-python prebuilt wheel on Windows, git on PATH,
  typer<0.24 pin): run from the clone's own env; if the env fight exceeds the budget, log the
  blocker honestly and stop (this unit is S-effort by definition).
- the 9 target plan files are living documents under concurrent sessions — insert lines via
  small anchored edits, re-grep before each (W-7).

## Next
Both 2026-06-10 reasoning docs are fully discharged: six new lanes planned (J-1..J-6), the intake
chores gated and logged (J-7), the declines recorded. J-5's port inherits the resolved scanner.

End of J-7.
