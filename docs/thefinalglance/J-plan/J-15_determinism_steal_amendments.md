# J-15: determinism steal closeout — the source-cited amendments into existing plan files (the J-11 analog)

- Track J (determinism-steal extensions) | lever AUDITABLE | grounding STRONG (8-agent verified triage + 4 interface maps) | nature EXTENSION | effort S | WAVE 2 | zero SSB, zero runtime code

- Premise: the 2026-06-11 50-repo determinism triage (workflow/reasoning/steal-determinism-repos-2026-06-11.md) produced, besides the three build units (J-12/J-13/J-14), a set of small mechanism steals whose correct home is an amendment block inside an EXISTING plan file (GD-1: execute existing plans, never re-author), plus DO-NOT-COPY exhibits and ledger entries. J-15 discharges them exactly like J-11 did for the Arbor list.

## Steal/build (the amendment list — each block is source-cited, inserted in place)
1. I-34 (CRANE hybrid + grammar pinning): ADD the llguidance Rust leg — llguidance crates.io v1.7.6 (MIT, 525K dl, anti-slopsquat verified live 2026-06-11 via crates.io API; Moskal/guidance-ai) as the orchestrator-side grammar compile+validate+blake3-pin engine: ParserFactory at plan time = fail-fast malformed-grammar detection + the pin computed in the same blake3 stack as the RRA; sidecar XGrammar UNCHANGED for inference-time masking (Option A; Option B full Rust-side masking REJECTED — requires a sidecar inference-loop redesign). Compiled parser state is NOT serializable — recompile from spec each boot (cheap, README-verified). NEW DEP: requires explicit user confirmation at I-34 execution (house rule); xgrammar-rs rejected as the alternative (C++ binding, 5.6K dl vs 525K).
2. E-31/H-7 (calibrated routing): ADD the recorded-escalation-hops idiom — every attempted+failed backend hop in a fallback chain lands its own journal row, not just the final RouteDecisionEvent (the intentusnet WAL pattern); prefer-reuse the existing RouteDecisionEvent kinds, decide at edit.
3. J-2 (elai-as-MCP-server): ADD two conductor patterns — (a) script-step-with-exit-code-routing + LLM-free set-step as BSP node shapes if J-2 grows workflow verbs; (b) MCP tool-config DELEGATION to the host runtime (never embed MCP server config in elai workflow definitions). Plus the intentusnet transparent-proxy shape note (client -> gateway -> server with zero changes either side) for the recording leg.
4. D-11 (constructive-checker registry): ADD the Logic-LLM solver-error-feedback loop — on checker FAIL, the solver's error message is fed back for ONE bounded reformulation pass before the claim is ruled UNGROUNDED (teacherpeterpan/Logic-LLM pattern, agent det-repos-H4); bounded, deterministic routing, the checker verdict stays load-bearing (TD-3).
5. H-49 (reversible edits/time-travel): ADD (a) event_id-anchored fork points — forks key on stable event ids, never positions (positions do not survive replay reordering; the culpa finding); (b) the mutate-then-replay affordance as an adversarial-testing rider (modify a recorded response, replay downstream — pairs with J-14's fixture format).
6. I-49 (bedrock/azure replay-fixture harness): ADD the fixture contract line — fixtures come from J-14's `rra export-fixture` format; matching key (model + message-content-blake3 + tool_name), NEVER positional.
7. DO-NOT-COPY exhibits (into the steal ledger + the H-49/I-49 blocks): culpa stores auth headers verbatim in session JSON (the J-14 scrub is the counter-design); culpa/evalcraft positional stub-matching is brittle by construction; Arbor-class fail-opens already recorded by J-11 stand.
8. REJECTED/ALREADY-TRIED ledger upserts (feature map Section 4): the ~40 triage rejections in one block (constrained-gen cluster incl. the EMPTY eth-sri/domino repo; instructor/validation cluster; workflow/MCP cluster; LayerCast/TBIK Hopper-only-TP-irrelevant; eval-harness cluster; Anandb71/arbor = name-collision, a dep-graph tool redundant with DKG+B-29 — NOT the ledger Arbor) so no future steal session re-litigates them.

## Existing ground truth
- target files exist and are PLANNED/unexecuted (I-34, I-49, H-49 partial, J-2, D-11 dark, E-31/H-7 partial) — amendments are insertions, not re-authoring; each block carries its agent/interface-map citation.
- triage ledger: workflow/reasoning/steal-determinism-repos-2026-06-11.md (agent ids + verdicts). interface maps under workflow/papers/ (life-harness x2, detllm, batch-invariant-ops).

## Integration
- amendment blocks into: docs/thefinalglance/I-plan/I-34*, I-49*, H-plan/H-49*, J-plan/J-2*, D-plan/D-11*, and the E-31/H-7 routing file(s) — exact filenames glob-resolved at edit; + workflow/reasoning/J-15-decisions.md; + the feature-map ledger upserts (Section 4 + PLANNED entries done by this authoring session).
- NO new action_type — zero SSB surface (documentation/amendment unit; JC-3 analog).
- frontend_pair: N/A — doc unit; justified.

## WIRING_CONTRACT
produces_symbols:
- source-cited amendment blocks in I-34 / I-49 / H-49 / J-2 / D-11 / E-31-or-H-7 plan files
- workflow/reasoning/J-15-decisions.md
consumes_symbols:
- workflow/reasoning/steal-determinism-repos-2026-06-11.md (the verdict ledger)
- workflow/papers/{life-harness-paper,life-harness-source,detllm-source,batch-invariant-ops-source}-interface.md (W-3 ground truth)
reachable_from:
- the consuming plan files' own executors (each amendment is read at that session's mandatory-reads step)  [PLAN-DOC surface; JD-1 analog applies]
stub_ban:
- NO uncited amendment (every block names its agent return or interface map + the upstream file/URL)
- NO amendment that re-authors a target session's scope (insertions only)
- NO presenting a DO-NOT-COPY exhibit as a pattern to copy
observable_effect:
- trigger: "grep each target plan file for its amendment marker after insertion"
- expect_artifact: "each block grep-present with its source citation (I-34 = llguidance + dep-confirmation flag; H-49 = event_id forks + mutate-then-replay; D-11 = solver-feedback bounded pass; I-49 = fixture contract; J-2 = conductor patterns); the feature-map REJECTED ledger carries the triage block. NEGATIVE (R2): an uncited block, or one missing the new-dep user-confirmation flag on I-34, FAILS review."

## Grounding
- agent returns det-repos-S1/S2/S3/H1..H5 (8 agents, URLs fetched and listed per R-5; 2026-06-11) + p2p-ingest-1..4 interface maps.
- llguidance anti-slopsquat: crates.io API fetch 2026-06-11 (v1.7.6, MIT, 525K total dl) — re-verify at I-34 execution per house rule.
- steal ledger: workflow/reasoning/steal-determinism-repos-2026-06-11.md.

## COMPOSES-WITH
I-34, I-49, H-49, J-2, D-11, E-31/H-7, J-14 (fixture format + scrub counter-design), the feature-map maintenance contract. Restates none.

## Open questions / risks
- E-31 vs H-7 as the hop-recording home: both touch routing; pick the one whose executor lands first (decide at insertion, note in J-15-decisions.md).
- I-34's llguidance dep stays PENDING-USER-CONFIRMATION until that session executes; the amendment records the verification snapshot so the executor re-verifies, never trusts it.

## Next
Closes the 50-repo triage: every surviving mechanism has a home, every rejection is ledgered, nothing re-litigates.

End of J-15.
