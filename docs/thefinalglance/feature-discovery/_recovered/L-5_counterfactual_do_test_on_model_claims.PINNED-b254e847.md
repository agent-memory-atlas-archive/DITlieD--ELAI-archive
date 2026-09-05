# L-5: Counterfactual do-test on model claims — perturb the cause, observe the effect [KEYSTONE, CANT-LIE]

STATUS: PLANNED (WAVE 2; M-L; EXTENSION(F-4/C-28/C-29); KEYSTONE — the causal core, the un-forgeable signal)

Objective: the paper's strongest signal is INTERVENTIONAL — the action-marginalization test perturbs the action and measures whether the output responds (§4.1.ii); a model that ignores the intervention is hallucinating. The elai-lang counterfactual-causal-learning idea is identical: disable the source, re-run, see if the target changes. L-5 ports this to model CLAIMS: when a model emits a Causal/Behavior claim ("A causes/fixes B", "this patch fixes the timeout"), the harness RUNS the intervention — revert only A / disable the flag / patch only the claimed function / add a trace to the claimed path — then re-runs the F-4 cant-lie outcome (blake3 stdout-hash the model cannot forge), the C-28/I-20 mutation, or the C-29 metamorphic relation, and compares. A claim is labelled "causal-supported" ONLY when the interventional outcome moves as claimed (HG-1); static/temporal/import-edge evidence is scored at its weaker tier and never called causal (the LS-2 fake-precision guard). This is the un-forgeable predictor that feeds L-2 as P-counterfactual. GD-1: do NOT re-implement F-4 / the mutation engine / the metamorphic harness — drive them on the claim.

Estimated effort: ~1 session (~M-L). The intervention re-runs tests/mutations; in-process where the test harness is local, RunPod-gated only where a claim's repro needs the live model.
Pre-requisites (re-grep live per W-7): F-4 AntiFabricationGate.check() (task_runner.rs:9565, blake3 stdout_hash on TestOutcomePayload); C-28 mutation gate + I-20 budgeted scheduler (MutationBudgetedRun); C-29 IntentPair + MetamorphicRelation; CSQ (phase-2D); J-16 blast-radius cone (blast_radius.rs); G-1 taint; L-1 ClaimSet; L-2 PredictorSignal.
Parallel-safe with: L-3, L-4 (disjoint files; all feed L-2). Depends L-1.
Source: cgg-hallucination-causal-system-2026-06-27.md S5 + paper §4.1.ii/§5 (action-shuffle ratio) + the elai-lang counterfactual screenshot + L-CGG-GROUNDING-MAP NN-4/TS-7/TS-8/TS-9. Doctrine: L-0_INDEX.md (HG-1, the keystone delta).

## Context
Why this matters: round-trip (L-3) and semantic entropy (L-4) are uncertainty signals; the do-test is the only CAUSAL one, and it is the only signal the model cannot game (F-4's stdout-hash is captured by the orchestrator, not the worker — wiring R8). The paper measures "action shuffle ratio": one-step flow MSE under the true action vs a batch-shuffled action; ratio <= 1.1 means the model IGNORED the action = hallucination (§5). The LLM port: outcome under the claimed intervention vs outcome without it; if reverting A does NOT change B, the model's "A causes B" is marginalized/hallucinated. HG-1 is the discipline this unit enforces for the whole CGG: the word "causal" is reserved for interventional/executable evidence. The honest residue (GD-5): not every claim is cheaply interventional (some need production traffic / non-snapshottable effects); those are scored at the runtime-trace tier (strong but not counterfactual) and the limit is named.

## Existing state (C/F/I/J coverage)
- F-4 test-integrity cant-lie [WIRED, task_runner.rs:9565] — AntiFabricationGate.check(); blake3 stdout_hash on TestOutcomePayload; auditor cannot emit passed=true without the orchestrator-captured hash. The un-forgeable outcome. [MAP]
- C-28 mutation gate [PARTIAL] + I-20 budgeted scheduler [WIRED] — perturb-and-observe; mutants on the diff cone; MutationBudgetedRun. [MAP]
- C-29 IntentPair + MetamorphicRelation [WIRED] — metamorphic relations wired unconditionally. [MAP]
- CSQ + inline feasibility [WIRED, phase-2D] — counterfactual safety questions on fs/net/money/auth. [MAP]
- J-16 blast-radius [PLANNED, blast_radius.rs] — the cone the perturbation touches; selects the regression set. [MAP]

## Reads (mandatory before coding)
1. _grounding/L-CGG-GROUNDING-MAP.md (NN-4, TS-7/8/9); cgg-hallucination-causal-system-2026-06-27.md S5; paper §4.1.ii + §5 (action shuffle ratio); the elai-lang counterfactual screenshot.
2. task_runner.rs:9565 (F-4 AntiFabricationGate); the C-28/I-20 mutation entrypoint; C-29 MetamorphicRelation; blast_radius.rs; L-1 ClaimSet + L-2 PredictorSignal.
3. F-0 W-1; G-0 GD-1/GD-5; CLAUDE.md TD-3, no-.unwrap, blake3; K-16 RR-1.

## Tasks (WAVE 2 — intervene on the claim, observe the outcome)
1. the intervention planner (~60 LOC): for a Causal/Behavior claim, derive the deterministic intervention from the claim refs + the J-16 cone (revert-only-A / disable-flag / patch-only-fn / add-trace). NO model call to design the intervention (TD-3 — derived from the typed claim + the cone).
2. the do-test runner (~70 LOC): run the F-4 outcome (or C-28 mutation / C-29 metamorphic) WITH and WITHOUT the intervention; compare the blake3-bound outcomes. Emit a P-counterfactual PredictorSignal{Counterfactual, moved: bool, delta}. causal_supported (in L-2) is set TRUE only when the outcome moves as claimed (HG-1).
3. the tier guard (~30 LOC): a claim with only static/temporal/import evidence is scored at that weaker tier and NEVER labelled causal (HG-1 — the fake-precision guard). Honest residue: non-snapshottable claims are scored at the runtime-trace tier, the limit recorded.
4. wire into L-2's cost-gated predictor call (the most expensive predictor — runs last, only on high cheap+P1+P3 risk) (~20 LOC). default OBSERVE-only (GD-4).
5. workflow/reasoning/L-5-decisions.md.

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/claim/counterfactual.rs — the intervention planner + do-test runner + tier guard + P-counterfactual PredictorSignal
consumes_symbols:
  - F-4 AntiFabricationGate (task_runner.rs:9565 — the un-forgeable outcome; COMPOSE, never re-implement)
  - C-28/I-20 mutation; C-29 MetamorphicRelation; CSQ; J-16 blast_radius.rs (the cone); G-1 taint
  - claim/mod.rs::ClaimSet (L-1); claim/risk.rs::PredictorSignal + causal_supported (L-2)
reachable_from:
  - claim/risk.rs::score_claim_risk > counterfactual_probe (L-2 cost-gate, only on high cheap+P1+P3 risk) [PRODUCTION counterfactual predictor]
  - a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO claim labelled causal without an interventional/executable pass (HG-1 — static/temporal is the weaker tier)
  - NO model call to design the intervention OR to judge the outcome (TD-3 — derived deterministically; F-4 hash adjudicates)
  - NO re-implementation of F-4 / the mutation engine / the metamorphic harness (GD-1 P0)
  - NO .unwrap on prod; blake3 not sha2
frontend_pair:
  N/A with justification — feeds L-2's risk tile; the underlying F-4/mutation rows already surface. Justified.
observable_effect:
  trigger: "do-test a 'this patch fixes the timeout' claim by running the failing repro with vs without the patch-only intervention (real F-4 path); plus an 'A causes B' claim where reverting A does NOT change B"
  expect_ssb: "prefer-reuse F-4 TestIntegrityEvent / MutationBudgetedRun / MetamorphicRelation; else +CounterfactualProbed (Pattern-2; Kind: OutcomeMoved/OutcomeUnmoved/NotInterventional); baseline+0..1 from LIVE ~268/577 (re-grep, W-9)"
  expect_artifact: "the fix-claim is labelled causal-supported ONLY if the F-4-bound outcome flips; the 'A causes B' claim where reverting A leaves B unchanged is labelled NOT-causal; NEGATIVE: a claim supported only by 'A imports B' is scored static-tier, never causal (HG-1)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/L-plan/L-5_counterfactual_do_test_on_model_claims.md --run-trigger
rg "intervention|counterfactual|moved" .elai_cc/crates/orchestrator/src/claim/counterfactual.rs   # the do-test present
rg -ci "causal" .elai_cc/crates/orchestrator/src/claim/counterfactual.rs   # causal label gated on interventional pass (read the guard)
test -f workflow/reasoning/L-5-decisions.md
```

## ACCEPTANCE
- [ ] HG-1: a claim is causal-supported ONLY on an interventional/executable pass; static/temporal scored weaker, never causal.
- [ ] the F-4 stdout-hash adjudicates the outcome (the model cannot forge it); the intervention is deterministic (TD-3).
- [ ] GD-1: F-4 / mutation / metamorphic harness consumed, not re-implemented.
- [ ] GD-5: non-snapshottable claims scored at the runtime-trace tier; the limit named in-file.
- [ ] cargo check --workspace GREEN; auditor PASS on -p orchestrator. No new dependency.

## WIRING DONE-GATE (clause by clause)
- W-1: done = the live do-test flips a fix-claim's causal label by re-running the F-4 outcome under the intervention (not cargo-green).
- W-2: no flag (a predictor; the gate flag is L-7).
- W-3: extends BUILT F-4/mutation/metamorphic, not a summary-only port.
- W-4: frontend_pair N/A (feeds L-2); justified.
- W-5/W-10: in-process where the repro is local; RunPod-gated where the claim's repro needs the live model.
- W-7: task_runner.rs:9565 + the mutation entrypoint + blast_radius.rs re-grepped at edit.
- W-9: SSB prefer-reuse F-4/mutation rows or +1.
- GD-2: the moved/unmoved verdict is recomputable from the two F-4-bound outcomes under ELAI_SEED.
- GD-5: non-interventional claims tier-capped; limit named.

## Tests to add
- crates/contracts/tests/l5_counterfactual_contract.rs (~90 LOC): (a) a true fix-claim: the F-4 outcome flips under the patch-only intervention > causal-supported; (b) a false fix-claim: outcome unchanged > NOT-causal; (c) an 'A causes B' claim where reverting A leaves B unchanged > NOT-causal; (d) a claim supported only by 'A imports B' is scored static-tier, never causal (HG-1); (e) the intervention is derived deterministically (no model call); (f) the F-4 hash binds the outcome (a forged passed=true is rejected); (g) a non-snapshottable claim is tier-capped at runtime-trace with the limit recorded.

## Files touched
- CREATE: .elai_cc/crates/orchestrator/src/claim/counterfactual.rs (~160 LOC)
- MODIFY: claim/risk.rs (~20 LOC: wire P-counterfactual + causal_supported into the cost-gate)
- CREATE: crates/contracts/tests/l5_counterfactual_contract.rs (~90 LOC)
- CREATE: workflow/reasoning/L-5-decisions.md

## Citations / Grounding
- arXiv:2606.27326 §4.1.ii + §5 (action-marginalization / action-shuffle ratio <= 1.1 = action ignored) [EXTERNAL/DESIGN-RATIONALE]; the elai-lang counterfactual-causal-learning screenshot; cgg-hallucination-causal-system-2026-06-27.md S5.
- ELAI ground truth (L-CGG-GROUNDING-MAP NN-4/TS-7/8/9): task_runner.rs:9565 (F-4), C-28/I-20 mutation, C-29 MetamorphicRelation, blast_radius.rs (re-grepped at edit, W-7).
- CLAUDE.md: TD-3, no .unwrap on prod, blake3-only; HG-1 (causal-means-interventional); GD-5 honest residue.

## Open questions / risks
- Intervention SAFETY: reverting/patching to test a claim mutates the workspace; run inside the I-2 COW worktree / Tier-C isolation (never on the live tree), and never apply a destructive intervention (HG-1 do-test is observe-the-outcome, not ship-the-change). Decide the isolation seam at authoring.
- Cost: the do-test is the most expensive predictor; it runs LAST in L-2's cost-gate, only when cheap+P1+P3 already flag high. Budget it via J-84; cover deterministically in the L-10 fixture tier.

## Next
L-5 gives the CGG its un-forgeable causal signal and enforces HG-1 plan-wide. With L-3/L-4/L-5 feeding L-2, the convergence fuse is complete; L-6 acts on a High tier, L-7 turns Critical into block/abstain.

End of L-5.
