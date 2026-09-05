# J-92: bind the orchestrator's own dispatch order into the FSM — make "edit before context-compiler returned" structurally illegal

- Track J (lacuna-fill track) | lever GATE-THE-GUARD | grounding STRONG (live FSM/I-1/J-66/J-68/orchestrate-protocol grep + ELAI's own logged P0) | nature BUILD (FSM substates + capability tokens) | effort M | WAVE 2 | depends I-1 (directive-supervisor plane), J-66 (temporal monitor), J-68 (ElaiFSM.tla)
- The SHAPE-B one-level-up residue made concrete: ELAI gates the WORKER's tool access by FSM state, but the ORCHESTRATOR running the gate is itself ungated. This is ELAI's OWN logged P0 (the main LLM rationalizing past dispatch order; the orchestrate-protocol.md origin incident).

- Premise: the FSM gates the WORKER's tool access by state, but the ORCHESTRATOR's dispatch SEQUENCE (context-compiler before edit, auditor before memory-write) is enforced by a PROSE rule (orchestrate-protocol.md H-1..H-10) + a Stop hook — and ELAI's gotchas record this as the LIVE P0 (the main LLM repeatedly skipping dispatch order under "save tokens" rationalizations). A prose rule the controller can rationalize past is not a gate; the regress moved up one level (from worker to orchestrator) and stopped at prose.

## Steal/build (report 10 §1 — guarded FSM transitions + capability tokens + formal verification)
1. PROMOTE orchestrator dispatch to first-class FSM SUBSTATES with EvidenceCheck — "edit before context-compiler returned" becomes a STRUCTURALLY ILLEGAL transition (no valid evidence row), not a rule to rationalize past. The substate sequence (PARALLEL_BATCH -> EXECUTING -> AUDITING -> MEMORIZING) is the FSM, not the prose.
2. UNFORGEABLE per-phase CAPABILITY TOKENS (Prifti et al. INV2/INV3, design-rationale): an agent cannot invoke a step without the token the orchestrator only mints on the PRIOR phase's evidence. An Edit on plan-scope files requires the token minted only when the context-compiler + integration-explorer evidence rows landed; a test-run requires the token minted only at AUDITING entry; a memory-write requires the MEMORIZING token. The token is the structural realization of H-2/H-3/H-4.
3. I-1's directive-supervisor plane IS this architecture (generator emits Intent->Directive; only the process-separated supervisor executes + holds the tokens; the generator cannot reach gate code). Move orchestrate-protocol.md H-1..H-10 from PROSE into those gates.
4. J-68's ElaiFSM.tla + TLC PROVES the ordering invariant (no path reaches Edit-on-plan-scope without the prior-phase evidence); J-66's temporal monitor catches a live violation per committed row.
EXCLUDED: an LLM deciding whether a transition is legal (the EvidenceCheck is deterministic; TD-3); a forgeable/advisory token (unforgeable, minted on evidence only); re-authoring the FSM or I-1 (extend them, GD-1).

## Existing ground truth (grep-confirmed; re-grep at edit, W-7/W-9)
- FULL FSM [WIRED] 9 states + 7 DAIRA evidence gates + InteractiveGate/AutonomousGate EvidenceCheck + FsmSubstate (precedent Framing B-40, agent_substate AUDITING/MEMORIZING); crates/orchestrator; phase-3A.
- DIRECTIVE SUPERVISOR PLANE I-1 [PLANNED] generator emits PURE Intent->Directive; process-separated Supervisor is the ONLY side-effect executor, holds credentials; generator cannot reach gate code.
- SSB TEMPORAL-MONITOR ENGINE J-66 [PLANNED] past-time-LTL online monitors firing per committed row; +SsbMonitorVerdict.
- SELF-CODE FORMAL VERIFICATION J-68 [PLANNED] ElaiFSM.tla + TLC (Inv2/3/4, design-time BFS).
- orchestrate-protocol.md H-1..H-10 (the PROSE rule + Stop hook) — the live P0 origin.
- live SSB baseline ~268/577 (grep 2026-06-26).

## Integration
- crates/orchestrator FSM: add orchestrator-dispatch FsmSubstates (PARALLEL_BATCH / EXECUTING / AUDITING / MEMORIZING) with EvidenceCheck per transition + a capability-token mint/consume on the I-1 supervisor plane. Edit/test/memory-write gated by token possession.
- +OrchestratorDispatchEvent SSB (Kind: TokenMinted / TokenConsumed / IllegalTransitionRefused) — net 0..+1 (prefer-reuse a phase_transition Kind; decide at edit). re-read at edit (W-9).
- J-68 ElaiFSM.tla gains the ordering invariant (the proof); J-66 gains the live monitor (G-orchestration: edit-without-token-row = a temporal violation).
- FEATURE FLAG: ELAI_ORCHESTRATOR_FSM_TOKENS_ENABLE (kill-switch; OBSERVE-ONLY first per GD-4 — the illegal-transition is FLAGGED before it BLOCKS, so a legitimate USER_OVERRIDE path is preserved) + a wire-dark-allowlist row with promotion plan (W-2/R9).
- frontend_pair: the dispatch-substate + token state on the orchestrate-run / FSM surface; W-6-justified where absent.

## WIRING_CONTRACT
produces_symbols:
- crates/orchestrator FSM dispatch-substates + EvidenceCheck-per-transition (PARALLEL_BATCH/EXECUTING/AUDITING/MEMORIZING)
- crates/orchestrator capability-token mint_on_evidence / consume_for_action (on the I-1 supervisor plane)
- crates/orchestrator refuse_illegal_transition (no evidence row -> structurally illegal, not rationalizable)
consumes_symbols:
- the FSM EvidenceCheck + FsmSubstate machinery (extended, not reauthored)
- I-1 directive-supervisor plane (the token holder; not reauthored)
- J-66 temporal monitor + J-68 ElaiFSM.tla (the live + design-time proofs; not reauthored)
reachable_from:
- an `orchestrate` / `auto-orchestrate` run that attempts an Edit on plan-scope before the context-compiler evidence row landed  [PRODUCTION]
- a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
- NO LLM deciding transition legality (deterministic EvidenceCheck; TD-3)
- NO forgeable/advisory capability token (unforgeable, minted on an evidence row only)
- NO prose-only enforcement left as the gate (the substate + token IS the gate; H-1..H-10 move into it)
- NO BLOCK default before the FP/USER_OVERRIDE-friction rate is bounded on >=3 runs (GD-4; the legitimate override path is preserved)
- NO re-authoring of the FSM / I-1 (extend; re-implementing = GD-1 P0)
frontend_pair: dispatch-substate + token state on the orchestrate/FSM surface, or W-6-justified absence.
observable_effect:
- trigger: "an `orchestrate` run where the controller attempts an Edit on a plan-scope file BEFORE the context-compiler + integration-explorer evidence rows have landed for the sub-phase (the exact P0 incident)"
- expect_ssb: "IllegalTransitionRefused naming the missing evidence + the attempted action; on the legal path, TokenMinted at PARALLEL_BATCH completion then TokenConsumed at the Edit"
- expect_artifact: "the Edit is structurally refused without the token (the transition has no valid evidence row); a legitimate USER_OVERRIDE in the same message mints the token via the override path and is journaled as such. NEGATIVE (R2): a token claimed without its backing evidence row FAILS verification, never mints; the monitor pointed at a run with no dispatch rows FAILS its self-test, never passes; the regress bottoms out at proven code (J-68) or a human (USER_OVERRIDE), by design."

## Grounding
- report 10 §1 (guarded FSM transitions + capability tokens + formal verification); Prifti et al. INV2/INV3 prior-art NAMES (design-rationale, LFG-3). Done-gate = the P0 Edit is structurally refused on a real orchestrate run.
- substrate: FSM [WIRED] + I-1 [PLANNED] + J-66/J-68 [PLANNED] + orchestrate-protocol.md (the prose this replaces) — feature-map 1.3/1.6.
- the live P0: orchestrate-protocol.md WHY-THIS-FILE-EXISTS (s-orch-e33 + the prior incident).
- live SSB baseline ~268/577 (grep 2026-06-26).

## COMPOSES-WITH
FSM + InteractiveGate/AutonomousGate (the host EvidenceCheck), I-1 (the supervisor plane + token holder), J-66 (the live ordering monitor), J-68 (the design-time ordering proof), orchestrate-protocol.md H-1..H-10 (the prose lifted into the gate), J-52 SubagentStop evidence gate (the return-time sibling). Restates none.

## Open questions / residue (reasoning §7)
- THE REGRESS BOTTOMS OUT (honest, by design): "who mints the minter" terminates at PROVEN CODE (J-68's TLA+ proof of the mint logic) or a HUMAN (the USER_OVERRIDE path) — exactly where ELAI already terminates (L1-HardVeto). This is the structure of the problem (reasoning §0/§7), not a defect to engineer away.
- OVERRIDE PRESERVATION: a legitimate USER_OVERRIDE must mint the token (the protocol's E-1 legal skip) — the gate cannot brick a deliberate human override; that path is journaled, not blocked.

## Next
J-92 turns the orchestrate-protocol prose into structure; J-66/J-68 prove and monitor it. This is the SHAPE-B residue (gate-the-guard) closed at the orchestrator layer, bottoming out at proven code or a human.

## Wargame

### EXPECTED_OBSERVATION
- trigger: an `orchestrate` run where the controller attempts an Edit on a plan-scope file BEFORE the context-compiler + integration-explorer evidence rows have landed for the sub-phase (the exact P0 incident).
- expect_ssb: `IllegalTransitionRefused` naming the missing evidence + the attempted action; on the legal path, `TokenMinted` at PARALLEL_BATCH completion then `TokenConsumed` at the Edit.
- expect_artifact: the Edit is structurally refused without the token (the transition has no valid evidence row); a legitimate USER_OVERRIDE in the same message mints the token via the override path and is journaled as such. NEGATIVE (R2): a token claimed without its backing evidence row FAILS verification, never mints; the monitor pointed at a run with no dispatch rows FAILS its self-test, never passes; the regress bottoms out at proven code (J-68) or a human (USER_OVERRIDE), by design.
- ON_FAIL: if the kill-switch ELAI_ORCHESTRATOR_FSM_TOKENS_ENABLE is OFF, the illegal transition is observed and flagged (observe-only) rather than refused; the SSB still records the attempt so J-66's monitor can fire. If the EvidenceCheck itself is bypassed (gate code unreachable), abort to L1-HardVeto.

### FORK
- FORK: if I-1's process-separated supervisor plane is missing in production at ship time, the thin InProcess path ships alone (token mint/consume in-process, no process-separated token holder); the FORK resolves when I-1 lands and the token holder moves out-of-process. If both the InProcess and the I-1 supervisor paths are covered, FORKS: none.

### ABORT
- ABORT if an LLM is proposed as the transition-legality judge (the EvidenceCheck is deterministic; TD-3; stub_ban).
- ABORT if capability tokens are forgeable or advisory (unforgeable, minted on an evidence row only; stub_ban).
- ABORT if the FSM or I-1 would be re-authored rather than extended (GD-1 P0; host seams: `FsmSubstate` in fsm.rs, `EvidenceCheck` in autonomous_gate.rs, `InProcessSupervisor` in directive.rs).

### VERIFY
- off-agent: `cargo test j92_` (filter) over crates/orchestrator; `python3 .claude/tools/production_caller_check.py --symbols mint_on_evidence consume_for_action refuse_illegal_transition` over the changeset.
- falsification: delete the production call site for `refuse_illegal_transition`, watch the j92_ suite go RED; restore the caller, watch GREEN. Repeat for `mint_on_evidence` / `consume_for_action`.
- runnable: `cargo test --package orchestrator j92_` ; `python3 .claude/tools/production_caller_check.py --files crates/orchestrator/src/dispatch_fsm.rs`

### REDTEAM
- attack 1: mint a capability token without the backing context-compiler evidence row -> `mint_on_evidence` refuses, no token minted, `IllegalTransitionRefused` SSB. Must fail closed.
- attack 2: attempt an Edit on plan-scope with the kill-switch ON but no token in hand -> `consume_for_action` refuses, Edit structurally blocked. Must fail closed.
- attack 3: forge the token bytes (hand-craft a TokenMinted without going through `mint_on_evidence`) -> verification FAILS, the forged token never mints, the Edit is refused. Must fail closed.
- attack 4: rationalize past the gate ("save tokens" / "I already know the context") -> the deterministic EvidenceCheck does not consult the LLM; the transition stays illegal. Must fail closed.

### INTEGRATION_SMOKE
- drive a `#[cfg(test)]` / contracts test in crates/orchestrator that simulates an `orchestrate` Edit arriving before the context-compiler evidence row; assert the `refuse_illegal_transition` path fires and the Edit is structurally refused (no token, no side effect). Then drive the legal path (evidence row lands -> `TokenMinted` -> Edit -> `TokenConsumed`) and assert GREEN.
- frontend_pair: the dispatch-substate + token state on the orchestrate/FSM surface is W-6 / owner-ui-hold for this plan; the smoke does not cover the UI pair.

### Residual (document, do not block refine)
- J-66 SsbMonitorVerdict + J-68 ElaiFSM.tla stay compose-with, not this plan's ship gate for the structural token code; they prove and monitor the ordering invariant this plan installs.

End of J-92.
