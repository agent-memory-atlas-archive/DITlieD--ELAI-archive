# K-8: absorbable-unit audit + provider-check PRE_GATE — stop building what the providers will eat; tag every remaining efficiency-class unit and gate future ones

STATUS: PLANNED (WAVE KW-1; audit + a rule; KD-4; NO SSB; the kill list is a PROPOSAL, never auto-applied)

Objective: the documented 2024-2026 absorption record (caching, compaction, server-side memory, native
subagents, tool-search/deferred-loading, batch-50% — all absorbed into provider platforms; KS-3 report
17, KS-2 §1.2) makes building provider-native features a waste. This session does TWO things: (a) sweep
the remaining PLANNED/PARTIAL efficiency-class units and tag each ABSORBED / AT-RISK / DURABLE, producing
a kill/de-scope PROPOSAL for user confirmation; (b) ship the KD-4 PRE_GATE rule — any efficiency-class
unit re-verifies the provider's CURRENT native surface (WebSearch) before EXECUTING. Hard boundary: the
kill list is a PROPOSAL artifact for user decision — NO removal/de-scope without explicit user
confirmation (house rule); the rule does NOT auto-block, it adds a mandatory re-check step.

Estimated effort: ~1 session (~4-5 h). The sweep is a feature-map read + WebSearch per efficiency unit;
the rule is one .claude/rules/ file + a conditional-routing line.
Pre-requisites: ELAI-FEATURE-MAP.md (the unit inventory + the upsert target). The absorption record
(KS-3 report 17). WebSearch for each provider's current native surface.
Parallel-safe with: K-1, K-9 (disjoint). Runs EARLY (WAVE KW-1) so it stops wasted builds immediately.
Source: K-MASTER-BRIEF §3.2 K-8 [KS-1]; KS-3 report 17, KS-2 §1.2; grounding map KS-4, KD-4. Doctrine: K-0_INDEX.md.

## Context
Why this matters: cost engineering is the weakest moat and providers absorb it on a 6-18mo cycle (KS-2
§1.2); every hour spent building a now-native feature is wasted, and worse, ships a feature the provider
does better. Mechanism: (a) for each remaining efficiency-class unit (H-Track-1 H-1..H-9 residue;
Track-14 I-10..I-16 residue; E-2/E-25/E-32 legs; J-24 tool-context legs), WebSearch the relevant
provider's current native surface and tag ABSORBED (kill or de-scope to the instrument half) / AT-RISK
(re-check at build time) / DURABLE (verification-coupled, keep); output a kill/de-scope list the USER
ratifies. (b) a KD-4 PRE_GATE rule that binds every FUTURE efficiency-class plan to re-verify the native
surface before EXECUTING. The absorption record (KS-3 report 17) is the checklist.

## Existing state (A/B..J coverage)
- KS-4 ELAI-FEATURE-MAP.md — the living unit inventory + the upsert target (header 234/543 STALE per
  KC-5; re-grep at read).
- The efficiency-class units to sweep: H-Track-1 (H-1..H-9 caching/compaction/tool-defer residue),
  Track-14 (I-10..I-16), E-2/E-25/E-32 legs, J-24 (tool-context). Confirm live PLANNED/PARTIAL status
  from the feature map + the *-SUMMARY_INDEX files at sweep time.
- conditional-routing.md (where the KD-4 rule gets loaded for efficiency-class tasks).
- HD-1 (efficiency numbers measured per config) — the durable form of cost engineering.

## Reads (mandatory before coding)
1. _grounding/K-GROUNDING-MAP.md (KS-4, KD-4) + the absorption record (KS-3 report 17 digest in KS-2 §1.2).
2. ELAI-FEATURE-MAP.md (the efficiency-class unit rows — the sweep input + upsert target).
3. The H-Track-1 / Track-14 / E-2/E-25/E-32 / J-24 SUMMARY entries (the unit scopes — via the map, not
   raw plan-file reads).
4. .claude/rules/conditional-routing.md (the load point for the new rule).
5. CLAUDE.md no-removal-without-confirmation (the kill list is a PROPOSAL).

## Tasks (WAVE KW-1 — audit + rule)
1. Sweep each efficiency-class unit: WebSearch the relevant provider's CURRENT native surface (Anthropic/
   OpenAI/Google + the relevant SDK), tag ABSORBED / AT-RISK / DURABLE with the evidence (the search
   result + date). ~one row per unit.
2. Upsert the tags into ELAI-FEATURE-MAP.md (the unit rows gain an absorption tag); bump the map date.
3. Author the kill/de-scope PROPOSAL (workflow/reasoning/K-8-absorbable-unit-audit.md): the ABSORBED
   units + the recommended kill-or-de-scope-to-instrument-half, for USER CONFIRMATION. NOT auto-applied.
4. Ship the KD-4 PRE_GATE rule (.claude/rules/provider-absorption-pregate.md): any efficiency-class unit
   re-verifies the provider's current native surface (WebSearch) before EXECUTING; duplicating a native
   feature is a kill-or-de-scope. Add the conditional-routing.md load line for efficiency-class tasks.
5. NEGATIVE arm: an efficiency-class task that skips the provider re-check is flagged at audit; the rule
   loads on a live efficiency-class task (JD-1 evidence = the rule fires in a real session).
6. workflow/reasoning/K-8-decisions.md.

## WIRING_CONTRACT
produces_symbols:
  - workflow/reasoning/K-8-absorbable-unit-audit.md (the ABSORBED/AT-RISK/DURABLE kill/de-scope PROPOSAL)
  - .claude/rules/provider-absorption-pregate.md (the KD-4 rule)
  - the absorption tags upserted into ELAI-FEATURE-MAP.md
consumes_symbols:
  - ELAI-FEATURE-MAP.md (the unit inventory + upsert target; KS-4)
  - the absorption record (KS-3 report 17 — the checklist)
  - .claude/rules/conditional-routing.md (the load point)
reachable_from:
  - conditional-routing loads provider-absorption-pregate.md on an efficiency-class task [PRODUCTION
    harness rule-load]
  - the audit flags an efficiency-class task that skipped the provider re-check
  - a #[cfg(test)]/contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO auto-applied removal/de-scope (the kill list is a PROPOSAL; user confirms — house rule)
  - NO absorption tag without WebSearch evidence + date (anti-stale; KD-4)
  - NO rule that only documents without a conditional-routing load line (it must actually fire)
  - NO runtime code, NO SSB (audit + rule)
frontend_pair:
  N/A — harness doc + rule + feature-map upsert. Justified (W-6).
observable_effect:
  trigger: "an efficiency-class plan task starts EXECUTING (conditional-routing loads the rule)"
  expect_ssb: "none (harness leg; JD-1 evidence)"
  expect_artifact: "the feature-map rows carry the absorption tags (grep-provable); the rule file exists
    and conditional-routing loads it on a live efficiency-class task; the kill list is a user-decision
    artifact, NOT auto-applied; the NEGATIVE arm: a task skipping the provider re-check is audit-flagged"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/K-plan/K-8_absorbable_unit_audit_provider_pregate.md
test -f .claude/rules/provider-absorption-pregate.md
test -f workflow/reasoning/K-8-absorbable-unit-audit.md
rg -n "provider-absorption-pregate" .claude/rules/conditional-routing.md   # the rule is actually loaded
rg -n "ABSORBED|AT-RISK|DURABLE" docs/thefinalglance/ELAI-FEATURE-MAP.md   # the tags are upserted
test -f workflow/reasoning/K-8-decisions.md
```

## ACCEPTANCE
- [ ] KD-4: the PRE_GATE rule exists + conditional-routing loads it for efficiency-class tasks.
- [ ] every absorption tag carries WebSearch evidence + date (anti-stale).
- [ ] the kill/de-scope list is a PROPOSAL for user confirmation, never auto-applied (house rule).
- [ ] the feature-map upsert is done + the map date bumped.
- [ ] No runtime code, no SSB.

## WIRING DONE-GATE (clause by clause)
- W-1: done = the rule loads on a live efficiency-class task (JD-1 artifact), not "the file exists".
- W-2/W-3/W-4/W-6: N/A (harness leg); justified. W-5: the live rule-load is the trigger.
- W-7: each unit's provider surface re-verified at sweep time (WebSearch, dated). KD-4: the rule it ships.

## Tests to add
- a harness check (the rule appears in conditional-routing's load list; JD-1): no Rust contract (audit +
  rule). The kill list's correctness is the user review.

## Files touched
- CREATE: workflow/reasoning/K-8-absorbable-unit-audit.md (~80 LOC, the PROPOSAL)
- CREATE: .claude/rules/provider-absorption-pregate.md (~30 LOC, the KD-4 rule)
- MODIFY: .claude/rules/conditional-routing.md (~2 LOC: load the rule for efficiency-class tasks)
- MODIFY: docs/thefinalglance/ELAI-FEATURE-MAP.md (~one tag per efficiency-class unit + date bump)
- CREATE: workflow/reasoning/K-8-decisions.md (~30 LOC)

## Citations / Grounding
- KS-3 report 17 absorption record (caching/compaction/memory/subagents/tool-search/batch absorbed
  2024-2026; predicted next: verification/audit, org governance, trajectory memory). [EXTERNAL]
- KS-2 §1.2: efficiency tracks are perishable; the durable form is verification/effort/routing choices.
- ELAI ground truth: KS-4 (feature map), the efficiency-class unit inventory.
- CLAUDE.md: no-removal-without-confirmation (the kill list is a proposal), KD-4, anti-slopsquatting
  (verify the provider's surface, don't assume).

## Open questions / risks
- The sweep's provider-surface WebSearches must be DATED and re-runnable — a 2026-06 verdict goes stale;
  the KD-4 rule's whole point is that the re-check happens at BUILD time, not once here.
- Tagging a unit ABSORBED is a strong claim — require the native-surface evidence in the proposal row;
  AT-RISK is the honest default when the surface partially overlaps.
- The user confirms the kill list; do NOT delete or de-scope any plan unit in this session (house rule).

## Next
This stops the corpus from spending effort on now-native features and binds every future efficiency-class
plan to re-check first. It pairs with K-9 (the model-refresh cadence — the other half of "don't build
stale"). After it, the activation-over-authoring preference (K-10) has teeth.

End of K-8.
