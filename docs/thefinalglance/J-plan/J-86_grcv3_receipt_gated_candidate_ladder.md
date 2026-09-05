# J-86: GRC-v3 receipt-gated candidate ladder activation — un-dark I-4 as the candidate selector over the J-83 carrier, with the receipt-absent vote-fallback rung, early-stop, and PRM-as-Tier-2-under-receipt

STATUS: PLANNED (WAVE GRC-3; M; ACTIVATION(I-4 + D-2 + I-24 + I-26) — wire the dark candidate ladder under the receipt-first rule; depends J-83, sequence after J-84/J-85)

Objective: the user's "candidate system — the part the model is more sure about proceeds, the rest stays" is, precisely, the I-4 EVIDENCE-RANKED SEARCH LADDER (DARK, GS-6) driven over the J-83 carrier: branch candidate beliefstates on the SessionDag, close each with its DOMAIN receipt (D-10-routed), the receipt-passing candidate ADVANCES (commit+freeze), the rest STAY/backtrack. J-86 activates it under the trust-spine rule and adds the two grounded refinements the reasoning run surfaced: (1) the SELECTOR HIERARCHY, triply grounded by 2024-2026 papers — deterministic RECEIPT (best) > statistical VOTE over INDEPENDENT samples (receipt-ABSENT fallback ONLY, never for factual-recall which routes to retrieval) > LLM self-eval (BANNED, the stale part of ToT, the local-8B-can't-self-evaluate rider). The receipt-absent vote-fallback rung (IMPLICATION 8) is a genuine ADDITION — it stops the system silently falling back to the banned self-judge when receipts run out. (2) EARLY-STOP on receipt-convergence (IMPLICATION 9, CGES cuts samples ~69% at equal accuracy; gains saturate ~20 samples) — never fixed best-of-N. PRM is placed correctly as a Tier-2 step-scorer UNDER the Tier-1 receipt (IMPLICATION 27): activate the dark D-2 PPM sidecar + I-22 progress rewards for intermediate steps with no deterministic receipt yet, guarded by the I-24 isomorphic canary (search OVER-optimizes past ~400 samples, ROUND-3 ADD-6). Backtracking over the SessionDag + H-49 replaces a bespoke tree-search engine (ToT's only durable primitive is backtracking; the selector self-eval is banned). HARD BOUNDARY: the selector is RECEIPTS, never a vote-when-a-receipt-exists and never LLM self-eval (IC-6, DC-3, F-4, D-13); the vote rung is ONLY for receipt-absent independent-error tasks (factual-recall → retrieval, never a vote — shared training bias, "Consensus is Not Verification"); PRM is Tier-2-under-receipt, never the Tier-1 gate; NO MCTS-first (IC-6 — a ladder best-of-N→tournament→PUCT, the mass at the bottom rungs, full tree-search rare and classifier-gated J-84); it does NOT re-author I-4/D-2/I-24/I-26/H-49 (activates+composes, GD-1).

Estimated effort: ~1.5 sessions (~M). The I-4 activation under receipt-first + the vote-fallback rung + early-stop + the D-2 Tier-2 wiring under I-24 + backtrack over H-49. Pre-requisites (re-grep, W-7): J-83 (the carrier candidates branch from); I-4 search/ladder.rs run_best_of_n (:237/277) + SearchNodeEvent + i4_search_ladder_contract.rs (GS-6); D-2 PPM sidecar (port 8101) + I-22 ProgressRewardScored + I-24 CanaryInjector/PrmCanaryEvent (GS-9); I-26 (early-stop budget); SessionDag branch/merge + H-49 backtrack (GS-10); D-10 (the domain receipt router); I-7 (escalate when a candidate exhausts).
Parallel-safe with: none in-wave (layers on J-83; reads J-84's WIDTH tier + J-85's guard). Depends J-83; sequence after J-84/J-85.
Source: reasoning-engine-grc-v3-2026-06-25.md FINAL SYNTHESIS §3/§9 + IMPLICATION 7/8/9/13/14/15/27 + ROUND-3 ADD-6; _grounding/J-GRC-V3-GROUNDING-MAP.md GS-6/GS-9/GS-10 + DC-3. Doctrine: J-0_INDEX.md (W/GD/ID/RD/VD inherited; ADDS no delta family — IC-6 receipts-not-votes/ladder-not-MCTS, VD-4 stronger-than-generator, RD-1 abstention all cover it).

## Context
Why this matters: a naive "sample many reasonings and pick the confident one" reintroduces the self-grading failure the whole trust spine exists to prevent — and self-consistency majority-vote FAILS for factual claims (shared training bias) while a deterministic checker is STRICTLY superior when one exists (a wrong answer in 30/40 samples still fails the checker). So the candidate system is: branch candidate beliefstates → close each with its domain receipt → the receipt-passing candidate advances; votes are NOT the selector, and a vote is only the receipt-ABSENT fallback (and never for recall). ToT's value is mostly BACKTRACKING (60% of its gain), which ELAI already has (SessionDag + H-49) — so no bespoke tree-search engine, just receipt-gated branch-and-backtrack. Full candidate-branching sits HIGH on the J-84 ladder (hard tier), reached only when the classifier detects a branching space AND lower rungs failed their receipts; the mass is at the bottom (no-chain / single-chain). PRM (D-2) scores INTERMEDIATE steps that have no deterministic receipt yet, always UNDER the Tier-1 receipt, guarded by the I-24 canary against style-gaming.

THE LADDER (what J-86 activates):
  SL-1 SELECTOR HIERARCHY: deterministic RECEIPT (D-10-routed: F-4/D-11/J-1/H-trading) > VOTE over independent samples (receipt-absent, independent-error tasks ONLY) > LLM self-eval (BANNED). When a receipt exists it is the selector, full stop (DC-3, IC-6).
  SL-2 RECEIPT-ABSENT VOTE RUNG (the addition, IMPLICATION 8): when NO deterministic receipt exists (open-ended NL, no checkable answer), fall back to statistical consensus over INDEPENDENT samples — NEVER LLM self-eval, NEVER for factual-recall (which routes to retrieval, the confabulation case).
  SL-3 EARLY-STOP (IMPLICATION 9): branch until a receipt confirms one OR confidence converges OR the J-84 depth budget caps — never fixed best-of-N (CGES ~69% sample cut at equal accuracy).
  SL-4 PRM TIER-2-UNDER-RECEIPT (IMPLICATION 27): activate D-2/I-22 to score intermediate no-receipt-yet steps, ALWAYS under the Tier-1 receipt, guarded by I-24 (over-optimization past ~400 samples, ADD-6).
  SL-5 BACKTRACK (IMPLICATION 14): a candidate step's receipt FAILS → backtrack to the last receipt-passing J-83 entry over SessionDag/H-49 and branch a different continuation; no bespoke tree-search.

## Existing state (W-7 anchors; GS-* in the grounding map)
- I-4 EVIDENCE-RANKED SEARCH LADDER [DARK(ELAI_SEARCH_LADDER_ENABLE)] search/ladder.rs run_best_of_n (:237/277) + SearchNodeEvent + i4_search_ladder_contract.rs (branch_is_pruned_never_promoted) — the substrate J-86 activates (GS-6).
- D-2 PPM PROCESS REWARD SIDECAR [DARK(ELAI_PPM_ENABLE)] port 8101 + I-22 PROGRESS-SHAPED REWARDS [DARK] ProgressRewardScored + I-24 ISOMORPHIC CANARY [DARK] CanaryInjector/PrmCanaryEvent — the SL-4 Tier-2 + its anti-gaming guard (GS-9).
- SESSION DAG [WIRED] branch/merge + H-49 REVERSIBLE EDITS [PARTIAL] undo-tree/RV-1 — the SL-5 backtrack substrate (GS-10; no new tree-search engine).
- J-83 BeliefState carrier (this wave) — the candidates ARE branched beliefstates; D-10 select_grounding_mode (the SL-1 domain receipt router); I-26 (the SL-3 early-stop budget); I-7 (escalate when a candidate exhausts).

## Reads (mandatory before coding)
1. reasoning-engine-grc-v3-2026-06-25.md §3/§9 + IMPLICATION 7 (select-by-receipt) / 8 (vote-fallback rung) / 9 (early-stop) / 13 (no self-eval, 8B rider) / 14 (backtrack) / 15 (high-rung-gated) / 27 (PRM Tier-2-under-receipt); ROUND-3 ADD-6 (over-optimization cap).
2. _grounding/J-GRC-V3-GROUNDING-MAP.md (GS-6/GS-9/GS-10, DC-3).
3. search/ladder.rs IN FULL (run_best_of_n, the prune/promote) + i4_search_ladder_contract.rs + D-2/I-22/I-24 modules + SessionDag branch/merge + H-49 — re-grep live (W-7).
4. J-83 (the carrier candidates branch from) + J-84 (the WIDTH tier that triggers branching) + D-10 (the receipt router) + I-7/I-8 (escalate/abstain).
5. CLAUDE.md: IC-6 (receipts-not-votes, ladder-not-MCTS), VD-4 (stronger-than-generator), D-13 (cross-family, no self-critique gate), gate-manifest R8 (the I-24 canary fixtures frozen), no .unwrap on prod.

## Tasks (WAVE GRC-3 — activate the receipt-gated ladder)
1. Activate I-4 under receipt-first (~60 LOC): drive run_best_of_n over J-83 candidate beliefstates; the selector is the D-10-routed domain receipt (SL-1), never a vote-when-a-receipt-exists, never self-eval. Un-dark behind a kill-switch with an allowlist row (W-2).
2. The receipt-absent VOTE rung (~40 LOC, the addition SL-2): when no deterministic receipt exists AND the task is independent-error (NOT recall), fall back to statistical consensus over independent samples; recall routes to retrieval (never a vote).
3. Early-stop (~25 LOC, SL-3): stop branching on receipt-convergence OR confidence-convergence OR the J-84 budget cap; never fixed-N.
4. PRM Tier-2-under-receipt (~40 LOC, SL-4): activate D-2/I-22 for intermediate no-receipt steps, UNDER the Tier-1 receipt, guarded by the I-24 canary (frozen fixtures, R8).
5. Backtrack (~30 LOC, SL-5): on a candidate-step receipt FAIL, backtrack to the last receipt-passing J-83 entry over SessionDag/H-49; no bespoke tree-search.
6. workflow/reasoning/J-86-decisions.md (the selector hierarchy + the vote-rung-is-receipt-absent-only rule + the PRM-Tier-2-under-receipt placement).

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/search/ladder.rs (the receipt-first selector + the vote-fallback rung + early-stop) + the D-2 Tier-2-under-receipt + backtrack wiring (ACTIVATION over the dark substrate)
consumes_symbols:
  - I-4 run_best_of_n + SearchNodeEvent (ACTIVATE, do NOT re-implement; GS-6, GD-1)
  - D-2 PPM sidecar + I-22 ProgressRewardScored + I-24 CanaryInjector (the SL-4 Tier-2 + guard; GS-9)
  - SessionDag branch/merge + H-49 backtrack (the SL-5 substrate; GS-10)
  - J-83 BeliefState carrier (the candidates) + D-10 (the receipt router) + I-26 (early-stop budget) + I-7/I-8 (escalate/abstain)
reachable_from:
  - the J-84 WIDTH tier → branch candidates over J-83, select by receipt [PRODUCTION reasoning loop, hard tier]
  - a candidate-step receipt FAIL → backtrack over SessionDag/H-49 [PRODUCTION backtrack]
  - a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO vote-as-selector when a receipt exists (receipt is the selector; SL-1, IC-6); NO LLM self-eval as the selector (BANNED, DC-3)
  - NO vote for factual-recall (recall → retrieval; shared training bias)
  - NO PRM as the Tier-1 gate (Tier-2-under-receipt only; SL-4); NO uncapped candidate width (early-stop + the I-24 over-optimization cap; SL-3, ADD-6)
  - NO MCTS-first / bespoke tree-search (a ladder + SessionDag backtrack; IC-6); NO re-implementation of I-4/D-2/I-24/H-49 (activate; GD-1); NO .unwrap on prod
frontend_pair:
  the existing search-ladder / ConvergencePanel surfaces the candidate branches + the selected receipt + the early-stop point — decide the tile at edit (W-4); justified N/A if it rides the existing I-4/I-27 surfaces.
observable_effect:
  trigger: "drive the real reasoning entrypoint on a hard branching task WITH a deterministic receipt, and one with NO receipt (open-ended) (integration smoke, R10): assert receipt-selection on the first, independent-sample-vote on the second, recall→retrieval on a recall query"
  expect_ssb: "reuse SearchNodeEvent (I-4) / ProgressRewardScored (I-22) / PrmCanaryEvent (I-24) — net 0 from LIVE 268/577 (decide at edit if a Kind is needed, W-9)"
  expect_artifact: "the receipt-passing candidate advances and the rest are pruned-never-promoted (the I-4 invariant); a receipt-absent open-ended task uses an independent-sample vote (NOT self-eval); a recall query routes to retrieval (NOT a vote); a failing candidate backtracks to the last receipt-passing entry; the NEGATIVE arm: a candidate selected by an LLM grading its own paths MUST NOT happen, and PRM is never the Tier-1 gate"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-86_grcv3_receipt_gated_candidate_ladder.md --run-trigger
rg "receipt.*select|vote.*fallback|early.stop|backtrack|tier.2|under.*receipt" .elai_cc/crates/orchestrator/src/search/ladder.rs   # the SL-1..SL-5 legs
rg -c "self.eval|self.grade|model.*judges.*own|vote.*recall" .elai_cc/crates/orchestrator/src/search/ladder.rs   # MUST be 0 (no self-eval selector, no vote-for-recall; IC-6/DC-3)
rg -c "mcts|monte.carlo.*first" .elai_cc/crates/orchestrator/src/search/ladder.rs                                 # MUST be 0 (ladder not MCTS-first; IC-6)
python .claude/tools/production_caller_check.py --files .elai_cc/crates/orchestrator/src/search/ladder.rs
test -f workflow/reasoning/J-86-decisions.md
```

## ACCEPTANCE
- [ ] SL-1: when a receipt exists it is the selector; never a vote, never LLM self-eval (IC-6/DC-3).
- [ ] SL-2: the receipt-absent vote rung fires ONLY for independent-error tasks; recall routes to retrieval (never a vote).
- [ ] SL-3: early-stop on receipt/confidence convergence or the J-84 budget cap; never fixed best-of-N.
- [ ] SL-4: D-2/I-22 PRM is Tier-2-under-receipt, guarded by the I-24 canary (frozen fixtures, R8); never the Tier-1 gate.
- [ ] SL-5: a failed candidate backtracks over SessionDag/H-49 (no bespoke tree-search engine; IC-6).
- [ ] GD-1: I-4/D-2/I-24/H-49 activated not re-implemented; cargo check GREEN; auditor PASS -p orchestrator; production_caller 0-DARK; the un-dark flag has an allowlist row (W-2/R9).

## WIRING DONE-GATE (clause by clause)
- W-1: done = the hard-branching + receipt-absent + recall integration smoke run-trigger (each selects correctly), not file-exists.
- W-2: I-4/D-2 un-darked behind kill-switches with wire-dark-allowlist rows + promotion plan (R9); not a terminal default-OFF.
- W-3: ACTIVATION of the dark I-4/D-2 substrate under the receipt-first rule, not a summary-port; the new legs (vote-rung, early-stop) extend it.
- W-4: rides the I-4/I-27 ConvergencePanel (decide tile at edit) or justified N/A. W-5/W-10: in-process; D-2 PRM scoring is remote-sidecar-gated (the live PRM leg is a separately-gated checkbox, W-10).
- W-6: existing search/convergence panel or justified N/A. W-7: ladder.rs / D-2 / I-24 / SessionDag / H-49 re-grepped at edit.
- W-9: reuse SearchNodeEvent / ProgressRewardScored / PrmCanaryEvent; net 0 (a Kind add decided at edit, lockstep).
- IC-6: receipts-not-votes, ladder-not-MCTS. VD-4: the verifier (receipt) is stronger-than-generator + the I-24 canary guards the PRM. GD-2: candidate selection recomputable from SearchNodeEvent rows.

## Tests to add
- crates/contracts/tests/j86_candidate_ladder_contract.rs (~90 LOC): (a) the receipt-passing candidate advances, the rest pruned-never-promoted (I-4 invariant); (b) a receipt-absent independent-error task uses an independent-sample vote; (c) a recall query routes to retrieval, NEVER a vote; (d) an LLM-self-eval selection is rejected (DC-3); (e) early-stop fires on receipt convergence (not fixed-N); (f) D-2 PRM scores an intermediate step only UNDER the Tier-1 receipt and is canary-guarded (I-24); (g) a failed candidate backtracks to the last receipt-passing entry (SL-5); (h) over-width past the cap is stopped (ADD-6 over-optimization); (i) corrupt receipt input degrades to abstain/escalate, never a silent self-eval (R1).

## Files touched
- MODIFY: .elai_cc/crates/orchestrator/src/search/ladder.rs (~180 LOC: receipt-first selector + vote-rung + early-stop + Tier-2-under-receipt + backtrack)
- MODIFY: the D-2/I-22/I-24 activation seam (wire the PRM Tier-2 under the receipt, canary-guarded) (~40 LOC)
- MODIFY: .elai_cc/wire-dark-allowlist.toml (the I-4/D-2 un-dark rows: type/owner/expiry/promotion_plan_id/smoke) — and ssb.rs/.golden IF a Kind is added (lockstep)
- CREATE: crates/contracts/tests/j86_candidate_ladder_contract.rs (~90 LOC)
- CREATE: workflow/reasoning/J-86-decisions.md (~30 LOC)

## Citations / Grounding
- reasoning-engine-grc-v3-2026-06-25.md §3/§9; IMPLICATION 7 (select-by-receipt, the decisive grounding), 8 (the receipt-absent vote rung), 9 (early-stop, CGES), 13 (no self-eval selector + the 8B-can't-self-evaluate rider), 14 (backtracking is ToT's durable primitive = SessionDag/H-49), 15 (high-rung classifier-gated), 27 (PRM Tier-2-under-receipt); ROUND-3 ADD-6 (search over-optimizes past ~400 samples → the I-24 cap).
- ELAI ground truth (GS-6/GS-9/GS-10): I-4 search/ladder.rs run_best_of_n (:237/277) + SearchNodeEvent (DARK), D-2/I-22/I-24 (DARK), SessionDag branch/merge + H-49 (the backtrack substrate) — re-grepped (W-7).
- arXiv [EXTERNAL]: self-consistency 2203.11171 + "Consensus is Not Verification" 2603.06612 (receipts > vote; vote fails for factual/shared-bias), ToT 2305.10601 (backtracking = 60% of the gain, self-eval is the weak link), test-time-compute 2408.03314 + CGES 2511.02603 (early-stop / difficulty-gated high rungs), Cobbe (verifier accuracy declines past ~400 samples = over-optimization).
- CLAUDE.md: IC-6 (receipts-not-votes, ladder-not-MCTS), VD-4 (stronger-than-generator verifier), D-13 (no same-model self-critique gate), R8 (frozen canary fixtures).

## Open questions / risks
- The "independent-error vs factual-recall" classification for the SL-2 vote rung must be deterministic (reuse the J-84 TYPE axis / D-10 — recall is the retrieve lane, so SL-2 only ever sees non-recall): verify the routing forecloses vote-for-recall structurally, not by a runtime check that could be bypassed.
- D-2 PRM is a remote-sidecar leg (GPU); the Tier-2 scoring is W-10-gated (a deterministic fixture leg now, the live PRM checkbox separate). Do NOT let the ladder DEPEND on the PRM being up — receipts are Tier-1 and self-sufficient; PRM is an under-receipt refinement only.
- Sequencing: J-86 layers on J-83 (carrier) and reads J-84 (the WIDTH tier that triggers branching) + J-85 (the guard). Ship last in the wave. Activating I-4/D-2 is the load-bearing change — verify the dark substrate's shape before driving it (a wrong seam is a material fork: STOP + handoff).

## Next
J-86 turns the user's "candidate system" into the receipt-gated ladder over the J-83 carrier: branch candidates, select by external receipt (vote only when no receipt and never for recall), early-stop on convergence, PRM only as Tier-2-under-receipt, backtrack over the shipped SessionDag. It completes the GRC-v3 technical track (J-83 carrier + J-84 classifier + J-85 guard + J-86 selector). K-15 measures the whole stack's win; K-16's doctrine binds the receipts-not-votes selector hierarchy this embodies.

End of J-86.
