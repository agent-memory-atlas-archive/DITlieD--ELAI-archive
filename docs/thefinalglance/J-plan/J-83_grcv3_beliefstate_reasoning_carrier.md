# J-83: GRC-v3 BeliefState reasoning-state carrier — complete I-11 into the typed reasoning unit that survives a model/context boundary, gated per-step by an external receipt

STATUS: PLANNED (WAVE GRC-1; M-L; EXTENSION(I-11) — completes the condense-on-escalate seam into the unification carrier; KEYSTONE of the GRC-v3 wave, everything else needs it)

Objective: the reasoning run (workflow/reasoning/reasoning-engine-grc-v3-2026-06-25.md) establishes that GRC-v3 is UNIFICATION not invention, and that its one keystone is a typed BeliefState carrier: the unit of reasoning that crosses a model/tier/unload boundary as CLAIMS+RECEIPTS, never raw transcript. I-11 already ships the condense-on-escalate seam (BeliefState::new + emit_condensed_receipt at belief_state.rs, wired escalation_waterfall.rs:239, GS-1) — J-83 COMPLETES it into the carrier: each entry = {claim, grounding-receipt, abstention/confidence bit READ from H-39 (GS-4, never re-derived, RD-1), DKG provenance}; the per-step RECEIPT GATE makes "the part the model is sure about proceeds, the rest stays" concrete at the reasoning-step grain (least-to-most decomposition WITH a per-step external-receipt gate — a step is COMMITTED iff its domain receipt passes, else it abstains/retries BEFORE it can poison the next step, the L2M cascade flaw the trust spine fixes); commit-and-freeze is the elegant core (a receipt-confirmed entry is simultaneously grounded, compressed-safe, and frozen against the generator second-guessing it away — IMPLICATION 31, three payoffs in one operation). The carrier is UNIFORM; the cross-domain win (C4) is that the VALIDATOR is domain-routed (D-10 → code F-4/test, math D-11, visual J-1/B-24, trading H-17..H-29) — the reasoning is identical across domains, only the receipt that closes a candidate changes. HARD BOUNDARY it must NOT cross: the model NEVER sets the confidence/abstention bit or adjudicates a commit (it is read from a receipt — the self-correction blind spot means a model cannot reliably ground itself, three papers; F-4/D-13/TD-3); the carrier preserves RECEIPTS losslessly and treats reasoning PROSE as lossy-foldable (DC-4, the I-14 compression rider, GA-4); it does NOT re-author H-50 (GS-2, the cloud-cache-prefix / local-unload legs it RIDES), I-7 (GS-5, the escalation seam it threads through), or the domain validators (composed, GD-1); it is NOT a from-scratch build (DC-1, I-11 is PARTIAL).

Estimated effort: ~1.5 sessions (~M-L). The carrier completion + the per-step receipt gate + the I-7 threading + a multi-turn handoff integration smoke (R10). Pre-requisites (re-grep live, W-7): I-11 belief_state.rs (BeliefState::new / emit_condensed_receipt, escalation_waterfall.rs:239); H-39 abstention_score.rs (the bit source, GS-4); I-7 i7_escalation_cascade_contract.rs (LocalWin/Escalated/BatchRouted, the PRODUCTION_HOOK); H-50 context_policy.rs + live_cli.rs:50 (local unload / cloud prefix, GS-2); D-10 select_grounding_mode (the domain-validator router, GS-8); SessionDag branch/merge (the candidate substrate J-86 layers on).
Parallel-safe with: J-84/J-85 share no files (the classifier/guard read the carrier but land in budget/verify seams); J-86 layers on top (sequence J-86 after J-83).
Source: reasoning-engine-grc-v3-2026-06-25.md FINAL SYNTHESIS §1/§4/§8 + IMPLICATION 10/11/18/20/31; _grounding/J-GRC-V3-GROUNDING-MAP.md GS-1/GS-4/GS-5/GS-2/GS-8 + GA-1 + DC-1/DC-4. Doctrine: J-0_INDEX.md (W/GD/ID/RD inherited; ADDS no delta family — RD-1 abstention-bit-from-D-16/H-39, IC-16 bind-to-the-wired-spine, IC-6 receipts-not-votes all already cover it).

## Context
Why this matters: ELAI splits reasoning/testing/execution across DIFFERENT models and contexts (cloud generator vs local generator; auditor vs executor; per-role), and the local lane has NO cross-request KV reuse (the sidecar PrefixKVCache is bypassed on the streaming path, DC-2). So the reasoning STATE must be carried ACROSS a model boundary AND across a context unload — and carried as a TYPED, loss-controlled object, not a raw chain (raw chain is exactly what CoT/L2M externalize and what I-11 PABU is designed to REPLACE: smaller, evidence-keyed). Mechanism: a BeliefState is an ordered set of entries; each entry carries its claim, the EXTERNAL receipt that closes it (or NONE → it stays open/abstained), the abstention/confidence bit read from H-39, and DKG-keyed provenance. The per-step receipt gate is the GRC Verify step (phase-2D) reading the receipt before the next step commits. On the cloud lane the carrier sits behind the cache-stable prefix (H-50); on the local lane it is unloaded early and selectively reloaded (H-50) because local has no KV reuse to lean on. This is "carry reasoning across a boundary without spiraling" — it carries claims+receipts, not prose.

THE CARRIER SEMANTICS (what J-83 ships; the GRC-v3 keystone):
  CV-1 ENTRY SHAPE: {claim, grounding_receipt: Option<ReceiptRef>, confidence_bit: read from H-39 AbstentionScored (correct=+1 / abstain=0 / confident-wrong=-t/(1-t)), provenance: DKG key + source SSB seq}. The model emits the claim text; the engine fills the receipt + bit. The model NEVER writes the bit (TD-3, the blind spot).
  CV-2 PER-STEP RECEIPT GATE: a step is COMMITTED to the carried prefix iff its domain receipt PASSES (D-10 routes to the validator: F-4 test-integrity / D-11 sympy-z3 / J-1 pixel-diff / H-trading). A failing/missing receipt → the entry ABSTAINS (stays open), gets more depth, or escalates (I-7) — it is NEVER committed on the chain's own say-so (the L2M poison-cascade fix).
  CV-3 COMMIT-AND-FREEZE (triple payoff): committing a receipt-confirmed entry FREEZES it — the generator may not re-open it. This is simultaneously the grounding (receipt confirmed), the compression (committed = foldable, prose lossy, receipt lossless, DC-4), and the overthinking defense (frozen against the generator oscillating away from it, GA-3/J-85). One operation, three wins.
  CV-4 BOUNDARY TRANSPORT: on escalate (I-7) the carrier is condensed-then-handed (the existing seam) — typed entries cross, not raw history; on local unload (H-50) the carrier is the thing unloaded and selectively reloaded. The receipts cross at full fidelity; the prose is rederivable.
  CV-5 DOMAIN ROUTER (the C4 cross-domain answer): D-10 select_grounding_mode picks the validator per claim; the carrier is identical for a Unity-render bug, a Sharpe-ratio bug, and a codebase bug — only the closing receipt differs.

## Existing state (B/C/D/E/F/G/H/I coverage — W-7 anchors, executor re-greps; GS-* in the grounding map)
- I-11 PABU BeliefState [PARTIAL] belief_state.rs BeliefState::new + emit_condensed_receipt; wired escalation_waterfall.rs:239 (BeliefStateEventKind::Condensed); i11_belief_state_contract.rs. The condense seam is LIVE (GS-1) — J-83 completes the carrier ON it.
- H-39 ABSTENTION-AWARE SCORING [WIRED] abstention_score.rs parse_eval_log_outcome reads correct/wrong/abstain from the EVAL LOG (a receipt) + AbstentionScored SSB (218) — the CV-1 confidence-bit source, read never re-derived (GS-4, RD-1).
- I-7 CASCADING ESCALATION [WIRED] i7_escalation_cascade_contract.rs (fail_condenses_then_escalates); EscalationCascadeKind {LocalWin, Escalated, BatchRouted}; escalation_waterfall.rs:239 — the CV-4 transport seam + the PRODUCTION_HOOK (GS-5).
- H-50 REGIME-AWARE CONTEXT POLICY [WIRED] context_policy.rs wired live_cli.rs:50; cloud=cache-stable-prefix, local=window-minimization+unload-early — the CV-4 lane policies the carrier rides (GS-2; do NOT re-author).
- D-10 MODE-ROUTING PREDICATE [WIRED+LOCKED] select_grounding_mode (modes A-E), deterministic — the CV-5 domain-validator router (GS-8).
- GROUNDING VERIFICATION [WIRED] 6 methods FILE/SOURCE/TEST/WEB/API/REGISTRY + F-4 cant-lie + CosmosCheck/VisualArtifactCheck — the receipts that close a candidate.
- GRC STATE MACHINE [WIRED] GrcState {Observe..Verify..Converge} crates/reasoning/src/config.rs — the per-step receipt gate lives at the Verify step (CV-2).
- SESSION DAG [WIRED] branch/merge + 3-pass lossless trim — the candidate substrate (J-86 layers on); I-5 pristine injector PARTIAL (GS-3) + K-12 must-preserve atoms guard the commit.

## Reads (mandatory before coding)
1. workflow/reasoning/reasoning-engine-grc-v3-2026-06-25.md FINAL SYNTHESIS §1/§4/§8 + IMPLICATION 10/11/18/20/31 + ROUND-4 (the live-code verification).
2. _grounding/J-GRC-V3-GROUNDING-MAP.md (GS-1/GS-4/GS-5/GS-2/GS-8, GA-1, DC-1/DC-4, R-1..R-4).
3. belief_state.rs IN FULL + escalation_waterfall.rs around :239 (the seam to extend) + abstention_score.rs (the bit) + context_policy.rs (the lane policy) — re-grep all live (W-7).
4. select_grounding_mode (grounding_mode.rs:157) + the GrcState Verify step (crates/reasoning) — the gate's home.
5. J-0 W/GD/ID/RD deltas; CLAUDE.md (TD-3 LLM-advisory, RD-1 abstention-from-receipt, IC-6 receipts-not-votes, IC-16 bind-don't-restate, no .unwrap on prod, blake3-not-sha2 for the provenance key).

## Tasks (WAVE GRC-1 — complete the carrier)
1. Extend belief_state.rs: the typed Entry {claim, grounding_receipt: Option<ReceiptRef>, confidence_bit (from H-39), provenance (DKG key, blake3)} + the ordered carrier over them; KEEP BeliefState::new / emit_condensed_receipt (GD-1, do not re-implement). ~120 LOC.
2. Per-step receipt gate (CV-2) at the GRC Verify step: route the claim's domain via select_grounding_mode (GS-8) → run the validator → commit iff PASS, else mark abstained (read the H-39 bit, never set it). Deterministic; the model does not adjudicate (TD-3). ~80 LOC.
3. Commit-and-freeze (CV-3): a committed entry is immutable for the rest of the run (frozen flag); a generator attempt to rewrite a frozen entry is dropped + logged (the J-85 overthinking-defense leg consumes this). ~40 LOC.
4. Thread the carrier through I-7's condense-on-escalate (CV-4): the escalation hands the TYPED carrier (entries cross), not raw history — extend the existing escalation_waterfall.rs:239 seam (the PRODUCTION_HOOK). Local-unload rides H-50 (GS-2, no new policy). ~50 LOC.
5. workflow/reasoning/J-83-decisions.md (the keystone decisions: entry shape, the C4 one-carrier-domain-routed-validators framing, the commit-freeze triple-payoff).

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/belief_state.rs — the typed Entry + the per-step receipt gate + commit-and-freeze (the carrier completion over I-11's condense seam)
consumes_symbols:
  - belief_state.rs::BeliefState::new / emit_condensed_receipt (EXTEND, never re-implement; GD-1)
  - abstention_score.rs (the confidence/abstention bit READ per RD-1, never re-derived; H-39)
  - escalation_waterfall.rs:239 EscalationCascadeKind (the I-7 transport seam threaded; GS-5)
  - select_grounding_mode (the domain-validator router; GS-8) + the 6 grounding methods + F-4 (the receipts)
  - context_policy.rs (the H-50 cloud-prefix / local-unload lane policy the carrier rides; GS-2)
reachable_from:
  - the GRC Verify step (crates/reasoning) → the per-step receipt gate [PRODUCTION reasoning loop]
  - escalation_waterfall.rs:239 → the carrier crosses the model/tier boundary on escalate [PRODUCTION escalation, the named hook]
  - a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO model-set confidence/abstention bit (read from H-39; TD-3, the self-correction blind spot)
  - NO commit on the chain's own say-so (the receipt gates the commit; the L2M poison-cascade fix)
  - NO raw-transcript handoff (typed entries cross; the carrier is claims+receipts, not prose)
  - NO re-implementation of I-11/H-50/I-7/D-10 (compose; GD-1); NO LLM re-summarization compaction (HD-2); NO ciphers (IC-3); NO .unwrap()/.expect() on prod; blake3 not sha2 for provenance
frontend_pair:
  the existing context/trust telemetry panels (ContextStatePanel / the GRC history surface) read the carrier's committed/abstained counts via the existing SSB poll — decide the exact tile at edit (W-4); a per-entry receipt drawer is a later UX, not this slice. If no new tile: justified N/A (the carrier surfaces through existing context panels).
observable_effect:
  trigger: "a multi-turn task that escalates local→cloud mid-reasoning (integration smoke, R10, HARD input — a strict validator, a step whose receipt FAILS): drive the real entrypoint, escalate, and assert the carried carrier"
  expect_ssb: "prefer-reuse BeliefStateEvent (+Kind: EntryCommitted / EntryAbstained / CarrierHandedOnEscalate) — net 0..+1 from LIVE 268/577, decide at edit (W-9)"
  expect_artifact: "after escalation the cloud lane receives the TYPED carrier (committed entries + their receipts), NOT raw history; a step whose domain receipt FAILS is recorded ABSTAINED and is NOT committed (the NEGATIVE arm that MUST hold); a committed entry the generator later tries to rewrite is dropped+logged (commit-freeze)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-83_grcv3_beliefstate_reasoning_carrier.md --run-trigger
rg "grounding_receipt|confidence_bit|commit.*freeze|frozen" .elai_cc/crates/orchestrator/src/belief_state.rs   # the carrier entry + freeze present
rg -c "set_confidence|model.*sets.*bit|self.*confidence" .elai_cc/crates/orchestrator/src/belief_state.rs        # MUST be 0 (the model never sets the bit, TD-3)
rg "escalation_waterfall|EscalationCascadeKind" .elai_cc/crates/orchestrator/src/belief_state.rs                 # threaded through the I-7 seam
python .claude/tools/production_caller_check.py --files .elai_cc/crates/orchestrator/src/belief_state.rs         # 0 DARK (the carrier has a production caller)
test -f workflow/reasoning/J-83-decisions.md
```

## ACCEPTANCE
- [ ] CV-1: every entry carries {claim, receipt-or-none, H-39 bit, DKG provenance}; the bit is READ from H-39, never set by the model.
- [ ] CV-2: a step commits iff its domain receipt (D-10-routed) PASSES; a failed/missing receipt abstains, never commits.
- [ ] CV-3: a committed entry is frozen; a generator rewrite attempt is dropped+logged.
- [ ] CV-4: the carrier crosses the I-7 escalation boundary as typed entries (not raw history); local-unload rides H-50.
- [ ] CV-5 / C4: select_grounding_mode routes each claim to its domain validator (one carrier, domain-routed receipts).
- [ ] GD-1: I-11/H-50/I-7/D-10 composed, not re-implemented; cargo check --workspace GREEN; auditor PASS on -p orchestrator; production_caller_check 0-DARK.

## WIRING DONE-GATE (clause by clause)
- W-1: done = the multi-turn escalation integration smoke run-trigger (carrier crosses typed, failed-receipt abstains), not cargo-green / file-exists.
- W-2: no new feature flag (the carrier is on the live reasoning path); if gated, default-ON behind a *_DISABLE kill-switch with an allowlist row.
- W-3: EXTENDS the live I-11 seam, not a summary-port. W-4: frontend_pair via existing context panels (decide tile at edit) or justified N/A.
- W-5/W-10: in-process at reasoning time; the escalation smoke is the live trigger. W-6: existing panel or justified N/A.
- W-7: belief_state.rs / escalation_waterfall.rs:239 / abstention_score.rs / context_policy.rs / select_grounding_mode re-grepped at edit.
- W-9: prefer-reuse BeliefStateEvent (+Kind); net 0..+1 from LIVE 268/577 derived; 9-tuple lockstep if a Kind is added.
- GD-2: the committed/abstained verdicts are recomputable from the SSB stream. RD-1: the abstention bit is read from D-16/H-39, never re-derived. IC-6: receipts gate the commit, never a vote.

## Tests to add
- crates/contracts/tests/j83_beliefstate_carrier_contract.rs (~80 LOC): (a) an entry whose receipt PASSES commits; (b) an entry whose receipt FAILS/absent ABSTAINS and is NOT committed (the poison-cascade fix); (c) the model cannot set the confidence bit (it is read from H-39); (d) a committed entry is frozen (a rewrite is dropped); (e) escalation hands the typed carrier not raw history; (f) corrupt/empty receipt input degrades to ABSTAIN (fail-closed, R1), never to a silent commit.

## Files touched
- MODIFY: .elai_cc/crates/orchestrator/src/belief_state.rs (~280 LOC: Entry shape + per-step gate + commit-freeze + I-7 threading)
- MODIFY: .elai_cc/crates/orchestrator/src/escalation_waterfall.rs (~30 LOC: hand the typed carrier at :239)
- CREATE: crates/contracts/tests/j83_beliefstate_carrier_contract.rs (~80 LOC)
- CREATE: workflow/reasoning/J-83-decisions.md (~30 LOC)
- (SSB, if a Kind is added) MODIFY: ssb.rs + action_type_lock.rs + .golden (9-tuple lockstep; decide at edit)

## Citations / Grounding
- reasoning-engine-grc-v3-2026-06-25.md FINAL SYNTHESIS §1 (carrier) / §4 (commit triple-payoff) / §8 (domain validators); IMPLICATION 10 (L2M + per-step receipt gate = sure-proceeds/rest-stays), 18 (preserve receipts / fold prose), 20 (self-correction blind spot — the engine grounds the model), 31 (commit = grounding + compression + overthinking-defense).
- ELAI ground truth (GS-1/GS-4/GS-5/GS-2/GS-8): belief_state.rs (I-11 PARTIAL), abstention_score.rs (H-39), escalation_waterfall.rs:239 (I-7), context_policy.rs+live_cli.rs:50 (H-50 WIRED), select_grounding_mode (D-10) — re-grepped at edit (W-7).
- arXiv [EXTERNAL, grounding for WHY not an ELAI number]: least-to-most 2205.10625 (the accumulated-prefix-as-state + the poison cascade J-83 gates), Reflexion 2303.11366 / faithfulness (the self-correction blind spot → engine-grounds-the-model), self-consistency 2203.11171 + "Consensus is Not Verification" 2603.06612 (receipts > votes).
- CLAUDE.md: RD-1 (abstention bit from D-16/H-39), IC-6 (receipts-not-votes), IC-16 (bind to the wired spine), TD-3, HD-2 (no LLM re-summarization), blake3-not-sha2.

## Open questions / risks
- The exact ReceiptRef shape (a pointer to the SSB row that closed the claim vs an inlined verdict): prefer a pointer (GD-2 recomputable, keeps the carrier small); decide at authoring against the F-4/D-11/J-1 verdict row shapes.
- R-1 (seam risk): J-83 is the carrier; J-86 layers candidates on it and J-84/J-85 read it. Ship J-83 with its I-7 consumer + the multi-turn smoke FIRST (vertical slice), never the carrier alone — the integration smoke is the load-bearing proof (R10).
- DC-1: because I-11 is PARTIAL not absent, the temptation is to treat this as "done already". It is NOT — the condense seam exists; the typed-entry carrier + per-step receipt gate + commit-freeze do not. Verify the seam shape before extending (a wrong seam is a material fork: STOP + handoff, not a silent re-implement).

## Next
J-83 is the GRC-v3 keystone: the typed reasoning unit that carries claims+receipts across a model/context boundary and commits a step only on an external receipt. It unlocks J-84 (the classifier that sizes the reasoning that fills it), J-85 (the overthinking guard that protects its frozen entries), and J-86 (the receipt-gated candidate branches over it). The K-15 bench measures the win; the K-16 doctrine binds the receipts-not-traces rule it embodies. The cross-domain answer (one carrier, domain-routed validators) is the framing this makes real.

End of J-83.
