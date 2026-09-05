# J-93: the Tier-2 composition wiring fills — seven shipped/planned halves, wired with one deterministic gate each

- Track J (lacuna-fill track) | lever WIRE-THE-HALVES | grounding STRONG (live E-6/H-49/H-35/G-1/crates-memory/H-51/GRC/D-10/GoalDriftCanary/I-45/rules-3F/I-39 grep) | nature BUILD (wiring + one gate each) | effort M-L | WAVE 2 | depends J-87 (E1 completeness, for the L-93-F outcome labels)
- Grouped per reasoning §10 ("group the Tier-2 wiring fills", fewest files). SEVEN legs, each a SHAPE-A composition gap (both halves shipped/planned, never wired). Each leg is its own vertical slice (producer + in-plan consumer + smoke); the file is the wave container (J-52 precedent). Each leg is OBSERVE-ONLY first (GD-4) and per the residue ledger names its monitor.

## LEG A — C3-AGENTS in-loop semantic supervisor (E-6 MAST on the live SSB stream)
- Lacuna: Erlang supervision (max_restarts=3) restarts on process DEATH; MAST shows most failures are SEMANTIC (alive-but-off-task). ELAI runs the E-6 MAST 14-mode classifier POST-HOC.
- Fill (report 10 §2): wire it as an IN-LOOP DETERMINISTIC supervisor over the live SSB action_type stream — FM-2.6 = stated-plan vs actual-tool-calls diverge; FM-1.3 = action_type sequence repetition (SEMANTIC, not byte); StuckLoop at 3 identical calls. Redirect/restart MID-RUN, not at audit. TD-3 SHARP: DETERMINISTIC pattern-matching on the action_type stream (FM-2.6/FM-1.3 are STRUCTURAL divergences, report 10 §2 "all rule-based"), NEVER an LLM watching the agent (who-watches-the-watcher).
- Wiring: extend E-6 MAST to consume the broadcast-after-commit SSB stream in-loop; reuse the MAST SSB +Kind. RESIDUE: poorly-calibrated monitors overload the human -> CONSERVATIVE thresholds, warn-before-halt.

## LEG B — C3-SESSIONDAG compensation registry (H-49 saga x H-35 /rewind)
- Lacuna: time-travel CANNOT undo external side effects (shell, network, git push, API) — the field's killer finding. H-49 has the saga LOG + RV-4 honest-irreversibility doctrine but NO compensation GENERATOR; it surfaces irreversibility without compensating.
- Fill (report 8 §uninvertible + Atomix prior-art): register a COMPENSATION per side-effecting tool where invertible (git push -> revert + force-with-lease guard; file create -> delete), explicitly MARK non-invertible for human; drive /rewind (H-35) to actually REVERSE invertible external effects, not just conversation/file state. Tier-C mediates + journals every side-effecting tool (the registry key) — and after J-87, completely.
- Wiring: a compensation table keyed by Tier-C tool; H-35 /rewind consumes it. reuse SessionRewound +Kind. RESIDUE: not all external effects invertible -> non-invertible STAYS surfaced-to-human (RV-4), never auto-reverted (cannot un-send an email; a DELETE may be unrecoverable).

## LEG C — C6-VIBEGATE taint that survives memory (G-1 x crates/memory x H-51)
- Lacuna: G-1's Provenanced<String> threads WITHIN a run; the label is LOST once a fact is stored to crates/memory and retrieved in a NEW session (persistent memory-poisoning, verified real on Bedrock).
- Fill: thread the G-1 Provenance THROUGH the memory write path (store the LABEL with the fact) + the H-51 cross-session handoff digest, so capability_gate sees the taint of a MEMORY-RETRIEVED fact.
- Wiring: G-1 Provenance persisted in crates/memory (E-5) write/retrieve + H-51 handoff; reuse FactValidationFailed / MemoryGovernanceEvent +Kind (J-21 sibling). RESIDUE: a fact ingested before this ships has no stored label -> a TTL/re-validation sweep re-labels on next retrieve (composes J-21/H-32 validate-before-use), and an unlabeled retrieved fact is treated as UNTRUSTED, never trusted-by-default.

## LEG D — C1-GRC per-step citation entailment (GRC x GROUNDMODES at step grain)
- Lacuna: GRC forces each step to cite a grounding, but the citation is SELF-ASSERTED (Correctness != Faithfulness — a cited source need not entail the claim).
- Fill (report 10 §3): route GRC's per-step grounding tags through C1-GROUNDMODES' DETERMINISTIC checker at STEP granularity (a type:source step gets its citation verified against the actual source span via the non-LLM SOURCE_CHECK) — the composition of GRC (forces citation) with GROUNDMODES (verifies it) that today runs only at CLAIM granularity. Natural home = the J-83 BeliefState carrier's per-step receipt gate (RR-1: a trace is never the confidence signal, only an external receipt is).
- Wiring: per-step SOURCE_CHECK on the GRC grounding tag, at the J-83 receipt gate; reuse the grounding-verification event +Kind. RESIDUE: SOURCE_CHECK covers source/file/test/api modes; a step grounded only by reasoning (no external receipt) is ABSTAIN-or-escalate, never silently passed (RR-1).

## LEG E — C3-AUTONOMOUS preference-drift vs capability-drift (GoalDriftCanary x I-45 sealed objective)
- Lacuna: GoalDriftCanary treats drift as capability/stagnation; 2026 research shows drift is partly a PREFERENCE conflict (an agent abandons a goal its trained values resist).
- Fill (report 10 §4): when drift fires, DETERMINISTICALLY diff WHICH constraint was dropped against the I-45 sealed typed objective; a dropped SAFETY/PRIVACY/SECURITY constraint escalates to a human as preference-drift (do NOT auto-retry — report 10's hard rule); capability-drift gets the persona-switch. I-45's sealed objective is the DETERMINISTIC foothold.
- Wiring: a deterministic dropped-constraint diff vs the I-45 seal in the GoalDriftCanary path; reuse the goal-drift SSB +Kind. RESIDUE: the safety/privacy/security constraint CLASS must be DECLARED (in the L1/J-89 spec-bundle) — an undeclared constraint cannot be diffed; the bundle's constraint taxonomy is the foothold, and an undeclared drop defaults to escalate (fail-safe), never silent auto-retry.

## LEG F — C5-RULES non-LLM ground-truth labels (the SSB outcome stream)
- Lacuna: the rules engine promotes on measured fpr/tpr/causal, but where do labels come from without an in-boundary LLM judge?
- Fill: source labels from the SSB OUTCOME stream — a BLOCK is a FALSE-POSITIVE if the SAME change later passed ALL deterministic gates (wiring/cant-lie/mutation/blast) and shipped with no downstream regression. The append-only journal IS the labeled outcome stream; wire rule-lifecycle promotion to it. CONSUMES J-87 completeness for the outcome record to be trustworthy.
- Wiring: a deterministic outcome-label deriver feeding rule-lifecycle fpr/tpr (phase-3F); reuse the rule-lifecycle SSB +Kind. RESIDUE: a label is only as trustworthy as the journal is complete (depends J-87); a change whose outcome record is incomplete is UNLABELED, not assumed-clean.

## LEG G — C9-PLANS auto-derive the feature-map's mechanical half (I-39)
- Lacuna: I-39 derives plan STATUS from the journal, but the feature-map's status counters + composition edges are HAND-UPSERTED (its header literally lists stale counters).
- Fill: auto-derive status flags (I-39 journal) + SCHEMA/TV (grep) + composition edges (elai.map/codegraph wiring contracts), leaving only the design-intent "why" hand-authored. Shrinks the stale-prone surface to the irreducible narrative. (K-22's only cross-track tie OUT-OF-K.)
- Wiring: a `.claude/tools/feature_map_derive.py` (stdlib) producing the mechanical half from I-39 + grep + elai.map; NO SSB (harness, JD-1 log evidence). RESIDUE: the derived half can DISAGREE with the hand-authored "why" — the tool FLAGS the disagreement (a status flag that contradicts the narrative), never silently overwrites the narrative.

## Existing ground truth (grep-confirmed; re-grep at edit, W-7/W-9)
- MULTI-AGENT FAILURE TAXONOMY E-6 [WIRED] MAST 14-mode + mast_cmd CLI. SESSION DAG / H-49 [WIRED/PARTIAL] saga log + /rewind H-35. TAINT G-1 [WIRED] + crates/memory E-5 [PARTIAL] + H-51 handoff [PARTIAL] + J-21 [PLANNED]. GRC phase-2A [WIRED] + GROUNDING VERIFICATION D-10 [WIRED] + J-83 BeliefState [PLANNED]. GoalDriftCanary C-10 [PARTIAL] + I-45 [PLANNED]. RULES ENGINE phase-3F [WIRED]. I-39 status-derivation [PLANNED]. live SSB baseline ~268/577 (grep 2026-06-26).

## Integration
- All seven legs reuse existing SSB action_types with new *Kind arms (PREFER-REUSE, net 0..+2 total across the file; decide per-leg at edit). Each leg AUDITOR-owned where it is a gate (H-3); each OBSERVE-ONLY first (GD-4) with a wire-dark-allowlist row + promotion plan where it adds a flag (W-2/R9).
- crates/orchestrator (legs A-F runtime) + .claude/tools (leg G harness). frontend_pair: legs A/B/E surface on the audit/canvas; leg G is a tool output. W-6-justified where absent.

## WIRING_CONTRACT (per-leg; each is a vertical slice)
produces_symbols:
- A: crates/orchestrator mast_inloop_supervisor (FM-2.6/FM-1.3/StuckLoop over the live SSB stream)
- B: crates/orchestrator compensation_registry + H-35 rewind_external (invertible-only; non-invertible surfaced)
- C: crates/memory persist_provenance + capability_gate_on_retrieved_taint
- D: crates/orchestrator per_step_source_check (at the J-83 receipt gate)
- E: crates/orchestrator dropped_constraint_diff (vs the I-45 seal; safety-class -> escalate)
- F: crates/orchestrator derive_outcome_labels (SSB outcome stream -> rule fpr/tpr)
- G: .claude/tools/feature_map_derive.py (status + SCHEMA/TV + edges)
consumes_symbols:
- A: E-6 MAST + the SSB broadcast stream | B: H-49 saga + H-35 /rewind + Tier-C tool journal | C: G-1 Provenance + crates/memory + H-51 | D: GRC tags + D-10 SOURCE_CHECK + J-83 | E: GoalDriftCanary + the I-45 seal + the J-89 constraint taxonomy | F: the SSB outcome stream + J-87 completeness + rules-3F | G: I-39 + elai.map/codegraph + grep
reachable_from:
- each leg from its real production seam on a live `elai` run (the in-loop supervisor on a drifting run; /rewind on an invertible external effect; a retrieved tainted memory fact reaching capability_gate; a per-step source citation; a goal-drift event; a rule-promotion tick; a feature-map upsert)  [PRODUCTION]
- a #[cfg(test)] / contracts/ caller NEVER satisfies any leg (R7)
stub_ban:
- NO LLM in any leg's decision (deterministic stream-match / diff / source-check / label-derive; TD-3 — leg A is the sharpest)
- NO auto-revert of a non-invertible external effect (leg B; RV-4 surfaced-to-human)
- NO trusted-by-default unlabeled retrieved fact (leg C; untrusted until re-validated)
- NO silently-passed reasoning-only step (leg D; abstain-or-escalate, RR-1)
- NO auto-retry on a dropped safety/privacy/security constraint (leg E; escalate, report 10 hard rule)
- NO outcome label from an INCOMPLETE journal record (leg F; depends J-87; unlabeled not assumed-clean)
- NO silent overwrite of the hand-authored narrative (leg G; flag disagreement)
- NO BLOCK default on any gate leg until FP bounded on >=3 runs (GD-4) + a wire-dark-allowlist row (W-2/R9)
frontend_pair: legs A/B/E on the audit/canvas surface; leg G tool output; W-6-justified where absent.
observable_effect:
- trigger: "A: a run whose stated plan says 'edit foo.rs' but whose tool calls touch bar.rs 3x (FM-2.6 divergence). B: a /rewind after a `git push`. E: a run that drops a declared PRIVACY constraint."
- expect_ssb: "A: the in-loop supervisor flags the plan-vs-action divergence MID-RUN (not at audit). B: a compensation (revert + force-with-lease) journaled for the push; a non-invertible effect marked surfaced-to-human. E: preference-drift escalation naming the dropped PRIVACY constraint."
- expect_artifact: "A: the divergence is caught before the run completes, warn-before-halt. B: the invertible push is actually reversed; the non-invertible one is surfaced, not auto-reverted. E: the run HALTS for human review, no auto-retry. NEGATIVE (R2): leg A pointed at a stream it cannot parse FAILS its self-test (never SKIP); leg F deriving a label from an incomplete record returns UNLABELED (never assumed-clean); leg G's derived status contradicting the narrative FLAGS, never silently overwrites."

## Grounding
- A: report 10 §2 (in-loop deterministic supervisor, "all rule-based"). B: report 8 §uninvertible + Atomix prior-art. C: persistent memory-poisoning (Bedrock-verified, carried from J-21). D: report 10 §3 (Correctness != Faithfulness; RR-1). E: report 10 §4 (preference vs capability drift; no-auto-retry hard rule). F: the append-only journal as the labeled outcome stream (depends J-87). G: I-39 + the feature-map MAINTENANCE CONTRACT. All DESIGN-RATIONALE only (LFG-3); each done-gate is the ELAI observable_effect above.
- substrate: E-6/H-49/H-35/G-1/E-5/H-51/J-21/GRC/D-10/J-83/GoalDriftCanary/I-45/rules-3F/I-39 (feature-map 1.2/1.3/1.5/1.7).
- live SSB baseline ~268/577 (grep 2026-06-26).

## COMPOSES-WITH
E-6 (leg A), H-49/H-35 (leg B), G-1/E-5/H-51/J-21 (leg C), GRC/D-10/J-83 (leg D), GoalDriftCanary/I-45/J-89 (leg E), rules-3F/J-87 (leg F), I-39/K-22 (leg G). Restates none.

## Open questions / residue (reasoning §7) — per leg above; cross-cutting:
- GATE-PROLIFERATION (R1): seven new gates raise the over-block rate. Each ships OBSERVE-ONLY until its FP is bounded on >=3 runs; the per-changeset gate-traversal budget caps how many fire on one change. Sequence by leverage (A and C first — supervision + memory-poisoning are the highest-value).

## Next
Legs C/F depend on J-87 (taint-through-memory needs the labels; outcome labels need completeness); leg D lands at the J-83 receipt gate; leg G is K-22's cross-track tie. Together they wire the SHAPE-A composition gaps the program identified.

End of J-93.
