# L-2: Unified ClaimRisk score + convergence fuse — the model-agnostic hallucination-risk object [KEYSTONE]

STATUS: PLANNED (WAVE 1; M-L; BUILD; KEYSTONE — the object every other L unit reads or writes)

Objective: ELAI's G-7 confidence producer already fuses logprob + self-consistency + generated-test agreement into a claim_score, but it is scoped to the agent's OWN output and gated DARK. L-2 generalizes it into the ClaimRisk object over ANY backend's claim (HG-3 black-box) and adds the paper's convergence principle (§4.2): a high/critical-risk verdict requires >=2 mechanistically-distinct predictors to AGREE — P1 round-trip residue (L-3), P2 reasoning oscillation (J-85, consumed not rebuilt), P3 semantic entropy (L-4). A single predictor may caveat but never blocks (HG-2 — this is what kills cost and false alarms). Cost-gate: the expensive predictors (L-4 resampling, L-5 do-test) fire ONLY after a cheap signal (L-1 refs-unresolved, P1 residue) flags medium risk, routed by the J-84 effort classifier. The output is a ClaimRisk tier {Low, Medium, High, Critical} per claim, driving L-6/L-7. HARD BOUNDARY: L-2 fuses + tiers; it does NOT decide the action (L-7) and does NOT itself run predictors (it orchestrates L-3/L-4/L-5). GD-1: generalize G-7 claim_score — do NOT re-implement it; reuse D-3 for the >=2-concur fuse.

Estimated effort: ~1 session (~M-L). In-process; the expensive predictor calls are RunPod-gated only where they invoke the local model (L-4/L-5 fixture tiers ship now).
Pre-requisites (re-grep live per W-7): G-7 claim_score + the trifecta SESSION gate (production_agent_rpc.rs:334); D-3 VerificationAggregator (>=2 concur); H-40 decorrelated retention; H-39 abstention-aware (AbstentionScored); D-16 tau; J-84 effort classifier; J-85 overthinking guard (oscillation = P2); L-1 ClaimSet.
Parallel-safe with: none in WAVE 1 (depends L-1; L-3/L-4/L-5 depend on this interface).
Source: cgg-hallucination-causal-system-2026-06-27.md S1/S7 + L-CGG-GROUNDING-MAP NN-2/TS-3/TS-5/TS-20/TS-22. Doctrine: L-0_INDEX.md (HG-1, HG-2, HG-3, ID-5).

## Context
Why this matters: the paper's deepest design move (§4.2) is that three mechanistically-distinct predictors AGREEING is itself the evidence the signal is real — redundancy is a feature. ELAI already encodes "convergence = trust" (D-3 >=2-concur, H-40 decorrelation) but only for verifier promotion, not for hallucination risk. L-2 is where that becomes a per-claim risk tier. HG-2 (>=2-agree) is the cost discipline: a single noisy predictor never blocks, so the expensive layer runs rarely (gated behind cheap medium-risk signals, J-84). HG-1 binds here too: a claim is only tiered "causal-supported" when L-5's interventional do-test passes; static/temporal evidence is scored at its weaker tier. The risk THRESHOLDS are ELAI-measured per L-10 (ID-5), never the paper's rho/AUROC.

## Existing state (D/G/H coverage)
- G-7 confidence producer [DARK(ELAI_CONFORMAL_REFUSAL_ENABLE), production_agent_rpc.rs:334] — fuses logprob+self-consistency+generated-test into claim_score; Platt ECE 0.32>0.03. The seed L-2 generalizes. [MAP]
- D-3 VerificationAggregator [DARK] — >=2 verifiers concur to promote; verifier_adapters default None. The convergence fuse. [MAP]
- H-40 decorrelated retention [PARTIAL] — prune only on low-lift AND low-unique-catch (Weaver). Keeps the predictors decorrelated. [MAP]
- H-39 abstention-aware scoring [PARTIAL] — confident-wrong=-t/(1-t); SSB AbstentionScored. The penalty term. [MAP]
- J-84 2-axis reasoning-effort classifier [PLANNED] — routes trivial/easy/hard; the cost-gate. [MAP]
- J-85 overthinking guard [PLANNED] — answer oscillation r=0.78 = P2 flow-instability, consumed as an input. [MAP]

## Reads (mandatory before coding)
1. _grounding/L-CGG-GROUNDING-MAP.md (NN-2, TS-3/5/6/20/22); cgg-hallucination-causal-system-2026-06-27.md S1/S7.
2. G-7 confidence producer (production_agent_rpc.rs:334); D-3 VerificationAggregator; H-39 AbstentionScored; D-16 tau; J-84 classifier interface; J-85 oscillation signal; L-1 ClaimSet.
3. F-0 W-1; G-0 GD-2/GD-4; K-16 RR-1 (trace never the confidence signal); CLAUDE.md TD-3/ID-5/blake3.

## Tasks (WAVE 1 — fuse, tier, cost-gate)
1. the ClaimRisk type + tiers (~40 LOC): ClaimRisk{tier: RiskTier{Low,Medium,High,Critical}, signals: Vec<PredictorSignal>, causal_supported: bool}; PredictorSignal{source: PredictorKind{RoundTrip,Oscillation,SemanticEntropy,Counterfactual,LogprobOptional}, value, fired: bool}.
2. the convergence fuse (~70 LOC): generalize G-7 claim_score over a ClaimSet; tier = function of how many distinct predictors fired AND agreed — HG-2: High/Critical require >=2 distinct predictors to agree; a single predictor caps at Medium. causal_supported set ONLY by L-5's interventional pass (HG-1). Logprob is an OPTIONAL signal (HG-3): absent on a backend that exposes none, the fuse still produces a valid tier.
3. the cost-gate (~50 LOC): cheap signals first (L-1 refs-unresolved + L-3 round-trip residue); the expensive predictors (L-4/L-5) invoked ONLY on medium-or-above cheap risk, routed by J-84. Emit the ClaimRisk row.
4. wire into the orchestrator post-extraction (~25 LOC): consume L-1's ClaimSet, produce a ClaimRisk per claim for L-6/L-7. Default behavior = OBSERVE-only (record, do not act) until L-7's gate flag.
5. workflow/reasoning/L-2-decisions.md.

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/claim/risk.rs — ClaimRisk / RiskTier / PredictorSignal + the convergence fuse + the cost-gate
consumes_symbols:
  - G-7 claim_score (production_agent_rpc.rs:334 — GENERALIZE, never re-implement)
  - D-3 VerificationAggregator (the >=2-concur fuse); H-40 decorrelation; H-39 AbstentionScored (the confident-wrong penalty); D-16 tau; J-84 effort classifier (cost-gate); J-85 oscillation (P2 input)
  - claim/mod.rs::ClaimSet (L-1)
reachable_from:
  - the orchestrator claim-intake seam > score_claim_risk [PRODUCTION risk-scoring]
  - a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO single-predictor High/Critical verdict (HG-2 — >=2 distinct predictors must agree)
  - NO logprob dependency (HG-3 — optional booster only; valid tier without it)
  - NO causal tier without L-5's interventional pass (HG-1)
  - NO LLM setting or overriding a tier (TD-3); NO re-implementation of G-7/D-3 (GD-1 P0); NO .unwrap on prod; blake3 not sha2
frontend_pair:
  a ClaimRisk tile on the trust/context panel (decide at edit, W-4) OR "N/A with justification — verdicts flow to the existing trust panels. Justified."
observable_effect:
  trigger: "score a ClaimSet containing one claim with P1-residue-high AND P3-divergent, and one claim with only a single predictor firing (real claim-intake seam)"
  expect_ssb: "+1 ClaimRiskScored (Pattern-2; Kind: LowRisk/MediumRisk/HighRisk/CriticalRisk) OR generalize G-7 rows; baseline+0..1 from LIVE ~268/577 (re-grep, W-9)"
  expect_artifact: "the two-predictor-agree claim scores High/Critical; the single-predictor claim caps at Medium (HG-2); both recomputable from the journal (GD-2)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/L-plan/L-2_claimrisk_score_convergence_fuse.md --run-trigger
rg "RiskTier|>= ?2|two_distinct|convergen" .elai_cc/crates/orchestrator/src/claim/risk.rs   # HG-2 fuse present
rg -ci "logprob.*required|require.*logprob" .elai_cc/crates/orchestrator/src/claim/risk.rs   # MUST be 0 (HG-3 optional)
test -f workflow/reasoning/L-2-decisions.md
```

## ACCEPTANCE
- [ ] HG-2: High/Critical requires >=2 mechanistically-distinct predictors to agree; single predictor caps at Medium.
- [ ] HG-1: causal_supported is set ONLY by L-5's interventional pass.
- [ ] HG-3: a valid tier is produced on a backend exposing no logprobs (black-box).
- [ ] TD-3: the LLM never sets/overrides a tier; the fuse is deterministic.
- [ ] GD-1: G-7 claim_score generalized + D-3 reused, not re-implemented.
- [ ] GD-4: ships OBSERVE-only (records tiers, does not act) until L-7's gate flag.
- [ ] cargo check --workspace GREEN; auditor PASS on -p orchestrator. No new dependency.

## WIRING DONE-GATE (clause by clause)
- W-1: done = the live claim-intake seam emits a ClaimRisk row with a real tier (not cargo-green).
- W-2: no behavior gate here (observe-only); the action flag lives in L-7.
- W-3: generalizes BUILT G-7 code, not a summary-only port.
- W-4: frontend_pair decided at edit or justified N/A.
- W-5/W-10: in-process; the expensive predictor calls it routes are L-4/L-5's RunPod concern.
- W-7: production_agent_rpc.rs:334 + the J-84/J-85 interfaces re-grepped at edit.
- W-9: SSB prefer-reuse G-7 rows or +1 from LIVE baseline.
- GD-2: the tier is recomputable from the per-predictor signals under ELAI_SEED.
- ID-5: the tier thresholds are ELAI-measured (L-10), never the paper's rho/AUROC.

## Tests to add
- crates/contracts/tests/l2_claim_risk_contract.rs (~90 LOC): (a) two agreeing distinct predictors > High/Critical; (b) one predictor firing alone > at most Medium (HG-2); (c) a valid tier with logprob signal absent (HG-3); (d) causal_supported false until an L-5 interventional signal is present (HG-1); (e) the cost-gate skips the expensive predictors on a Low cheap-signal claim; (f) the tier is recomputable from the recorded signals (GD-2); (g) the LLM cannot override a tier (TD-3).

## Files touched
- CREATE: .elai_cc/crates/orchestrator/src/claim/risk.rs (~160 LOC)
- MODIFY: the claim-intake seam (~25 LOC: score after extract)
- CREATE: crates/contracts/tests/l2_claim_risk_contract.rs (~90 LOC)
- CREATE: workflow/reasoning/L-2-decisions.md

## Citations / Grounding
- arXiv:2606.27326 §4.2 (convergence of distinct predictors = the signal is real) [EXTERNAL/DESIGN-RATIONALE, ID-5]; cgg-hallucination-causal-system-2026-06-27.md S1/S7.
- ELAI ground truth (L-CGG-GROUNDING-MAP NN-2/TS-3/5/6/20/22): production_agent_rpc.rs:334 (G-7), D-3, H-39 AbstentionScored, J-84, J-85 (re-grepped at edit, W-7).
- CLAUDE.md: TD-3, ID-5 (ELAI-measured thresholds), no .unwrap on prod, blake3-only; K-16 RR-1 (trace never the confidence signal).

## Open questions / risks
- The exact tier function (how predictor agreement maps to Low/Medium/High/Critical): the boundaries are ELAI-measured at L-10 (ID-5), seeded conservatively + OBSERVE-first (GD-4). Decide the seed function at authoring; never a paper constant.
- Predictor decorrelation: P1/P2/P3 must be genuinely distinct or the >=2-agree rule is fake convergence (the "fail-positive" risk). H-40 retention enforces decorrelation; verify at L-4 that semantic-entropy and round-trip do not collapse to the same signal.

## Next
L-2 is the spine of the CGG. L-3/L-4/L-5 each emit a PredictorSignal into it; L-6 acts on a High tier (targeted effort); L-7 turns a Critical tier into a block/abstain; L-10 measures whether the fused tier actually predicts hallucination on an ELAI gold set.

End of L-2.
