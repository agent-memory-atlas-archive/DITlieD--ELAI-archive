# J-67: multi-mode gate actions — a gate may suppress, insert, or edit, not only block-and-truncate

- Track J (NASA-gates wave) | lever CANT-DERAIL | grounding STRONG (edit-automata-interface.md W3, the 4-class hierarchy) / EXTERNAL (Ligatti 2005) | nature EXTENSION(J-12) | effort M | WAVE NASA-GATES-4 | depends J-66 (the verdict) + J-69 (journaled inserts/suppresses)
- Upgrades J-12's boolean Block(msg) to the Edit-Automata typed GateAction taxonomy.

- Premise: ELAI's pre-tool gates today only BLOCK (truncate the action). GROUNDED GAP: J-12's H2 pre-tool gate is "Block(msg)" = truncation-only; the edit-automata-interface.md (W3, Ligatti 2005) 4-class hierarchy adds three more enforcement modes — SUPPRESS (drop the one dangerous action, continue the run), INSERT (inject a mandatory remediation/approval step before the action), EDIT (= suppress + insert: redact a secret from a tool result, then proceed). A typed GateAction {Accept, Suppress, Insert, Block} lets the harness DO the safe thing instead of only killing. The hard constraint (W3 + the plan's RISK): Insert/Edit CHANGE the event stream, so every inserted/suppressed/edited action MUST be journaled as an SSB row — or replay/audit/RRA breaks (ties to J-69). And per ND-1 / W2, each policy's enforceability CLASS is classified BEFORE choosing its mode: a non-safety property cannot be promised "edit" semantics.

## Steal/build
1. a GateAction enum {Accept, Suppress(reason), Insert(Vec<RemediationAction>), Block(msg)} + a dispatcher in the PreToolUse / J-12 H2 chain. The existing gate chain (rules-engine 3F / hook chain 4C / J-12) returns a GateAction instead of bool; the orchestrator applies it.
2. SUPPRESS: drop the dangerous tool-call, emit a journaled "suppressed" row, continue the run (the agent self-corrects from the absence + the row).
3. INSERT: inject a mandatory step (an approval prompt, a declassify, a taint-screen) BEFORE the side effect; the inserted actions are journaled rows that appear in the SSB total order BEFORE the gated action.
4. EDIT (= suppress + insert): redact a secret from a tool RESULT (the H4 annotator surface) before it reaches the model, journaling the redaction.
5. ENFORCEABILITY CLASSIFIER (ND-1 / W2): before a policy is assigned a mode, a deterministic classifier tags it suppressible / insertion-enforceable / not-runtime-enforceable; a non-safety property is REFUSED "edit" semantics at authoring (the W3 caveat W5 guards).
EXCLUDED: an Insert/Edit whose actions are NOT journaled (breaks RRA replay — hard ban, ties J-69); promising edit-semantics for a liveness/non-safety property (ND-1); an LLM choosing the GateAction (the verdict comes from J-66's deterministic monitor or the rules engine, TD-3 — the LLM only RECEIVES the Block message and self-corrects); a second gate chain (extend J-12's, never fork it).

## Existing ground truth (PLAN-asserted unless noted; re-grep at edit, W-7/W-9)
- J-12 RuntimeInterventionHarness (PLANNED): H2 pre-tool protocol gate riding rules lifecycle.rs:1, evaluated in the PreToolUse chain at runtime_setup.rs:120; H4 per-tool result annotators (Option<String>, read-only) — the EDIT-mode redaction surface; +HarnessInterventionEvent. J-67 EXTENDS this, reusing its event and chain.
- rules engine 3F + hook chain 4C (F-1 / F-9 / E-30) — the existing PreToolUse gate chain that returns bool today; J-67 changes the return TYPE to GateAction.
- J-66 SsbMonitorVerdict (this wave) — the deterministic source of the verdict that selects the mode.
- J-69 chain-scan + the SSB single writer (this wave) — the journaling guarantee that makes Insert/Edit replay-safe.
- G-1 taint sets + trust_dialog.rs (CVE-2026-21852 gate) — the Insert-mode approval/declassify steps reuse these, never re-implement (W-8).

## Integration
- MODIFY the J-12 intervention dispatcher + the 3F/4C PreToolUse chain return type bool -> GateAction; NEW crates/orchestrator/src/intervention/gate_action.rs (the enum + apply dispatcher + the ND-1 enforceability classifier; ~200 LOC inside J-12's module). The inserted/suppressed/edited actions are written through the SSB single writer.
- NO new action_type — EXTEND J-12's HarnessInterventionEvent Kind set with Suppressed / Inserted / Edited arms (prefer-reuse; the inserted/suppressed actions are journaled under it). If J-12 has not shipped its event at edit, J-67 adds +GateActionApplied (baseline+1, re-grep W-9). re-read at edit.
- frontend_pair: the GateAction (suppress/insert/edit) rows on the J-12 intervention drawer (decide at edit) or a W-6-justified absence.

## WIRING_CONTRACT
produces_symbols:
- crates/orchestrator/src/intervention/gate_action.rs::GateAction (Accept/Suppress/Insert/Block enum)
- crates/orchestrator/src/intervention/gate_action.rs::apply (the dispatcher that enacts each mode + journals it)
- crates/orchestrator/src/intervention/gate_action.rs::classify_enforceability (ND-1 mode-eligibility classifier)
consumes_symbols:
- J-12 PreToolUse chain + HarnessInterventionEvent (the gate seam + the journaled event; EXTENDED not reauthored, GD-1)
- J-66 SsbMonitorVerdict (the deterministic verdict that selects the mode; TD-3)
- the SSB single writer + J-69 chain (the journaling guarantee for Insert/Edit)
- G-1 taint / trust_dialog.rs (the Insert-mode approval/declassify steps; reused, W-8)
reachable_from:
- a real `elai` run whose tool-call trips a policy resolved to Suppress/Insert/Edit, through the live PreToolUse chain  [PRODUCTION]
- a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
- NO Insert/Edit action that is not journaled as an SSB row (RRA-replay-breaking; hard ban, ties J-69)
- NO "edit" semantics promised for a non-safety property (ND-1 classifier refuses it)
- NO LLM selecting the GateAction (the verdict is deterministic, J-66/rules; the LLM only receives Block(msg), TD-3)
- NO second gate chain (extend J-12's; forking = GD-1 P0)
- NO silent Suppress (a dropped action ALWAYS leaves a journaled "suppressed" row; the dead path screams, R2)
frontend_pair: GateAction rows on the J-12 intervention drawer (decide at edit) or W-6-justified absence.
observable_effect:
- trigger: "a tool-call that a policy resolves to SUPPRESS (continues) and one that resolves to INSERT (an approval step), driven through the REAL PreToolUse path"
- expect_ssb: "HarnessInterventionEvent{Kind: Suppressed} for the dropped call (and the run continues); HarnessInterventionEvent{Kind: Inserted} with the approval/declassify rows appearing in the SSB total order BEFORE the gated side-effect row"
- expect_artifact (NEGATIVE, R1/R2): "an Insert whose remediation rows are NOT journaled FAILS the RRA-replay contract test; a policy classified non-safety that is assigned Edit FAILS the enforceability classifier; a Suppress that leaves no row FAILS (silent suppression is the banned mode); under ELAI_GATE_ACTION_DISABLE the chain falls back to J-12 Block-only and SAYS so (no silent behavior change)"

## Grounding
- W3 edit-automata-interface.md (Ligatti 2005, the 4-class enforcement hierarchy: truncation/suppression/insertion/edit; edit = suppress+insert) — the typed GateAction taxonomy source.
- W2 enforceable-security-policies-interface.md (Schneider 2000) — the ND-1 enforceability-class test that gates which mode a policy may use; W5 rv-survey (the 3-valued caveat that a non-runtime-enforceable property must not be promised edit semantics).
- substrate: J-12 RuntimeInterventionHarness (the Block-only gate this upgrades), the SSB single writer (the journaling spine).
- LIVE SSB baseline 268/577 (ssb.rs:62/:1508, grep 2026-06-22); net 0 (J-12 event extension) or +1 if J-12 unshipped at edit.

## COMPOSES-WITH
J-12 RuntimeInterventionHarness (EXTENDS its Block to {Accept,Suppress,Insert,Block}; reuses its chain + event), J-66 (the deterministic verdict source), J-69 (the journaling integrity guarantee Insert/Edit depends on), G-1 taint + trust_dialog.rs (the Insert-mode remediation steps, W-8 reuse), rules engine 3F / hook chain 4C (the PreToolUse seam). Restates none.

## Open questions / risks
- the RRA-replay contract is load-bearing: if an inserted action is not deterministically replayable from its journaled row, replay diverges — the journaled row must carry enough to re-enact the insert (decide the row schema with J-69 at edit).
- SUPPRESS changes agent behavior subtly (the agent sees an action vanish) — measure that the self-correction is real and not a silent stall (OBSERVE-ONLY first, GD-4); a suppressed-then-stalled run is a finding, not a success.
- mode/policy mapping is security-sensitive — the enforceability classifier is frozen (gate-manifest, R8) so an executing run cannot quietly upgrade a policy from Block to Edit on itself.

## Next
J-67 completes the runtime-enforcement spine (J-69 integrity -> J-66 verdict -> J-67 typed action); together they let the harness intervene proportionally instead of only killing, while every intervention stays journaled and RRA-replayable. The design-time verification half of the wave (J-68) and the assurance discipline (J-70) bound what these runtime gates are allowed to claim.

End of J-67.
