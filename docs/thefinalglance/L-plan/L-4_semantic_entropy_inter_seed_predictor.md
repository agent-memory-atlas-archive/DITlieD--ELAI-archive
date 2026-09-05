# L-4: Semantic-entropy inter-seed predictor (P3) — resample, cluster by meaning, measure disagreement [net-new]

STATUS: PLANNED (WAVE 2; M-L; BUILD — the one genuine net-new predictor in the CGG)

Objective: the paper's predictor 3 is inter-seed variance — N independent denoising trajectories that disagree mark epistemic uncertainty / scene-divergence (§4.2). The LLM analogue is SelfCheckGPT / semantic entropy (LS WV-2): resample N alternative answers, CLUSTER BY MEANING, and measure the entropy over meaning-clusters — high entropy = high hallucination risk. ELAI already resamples (I-4 best-of-N over SessionDag) and counts concurrence (D-3), but it does NOT cluster N samples by MEANING and emit semantic entropy as a named signal (DC-3 — the one genuine net-new producer). L-4 builds it: resample N via I-4, cluster via the F-12 DeterministicNliJudge (entailment-equivalence, TD-3 — NOT an embedding cosine, NOT an LLM-judge), emit semantic entropy as a P3 PredictorSignal into L-2. Cross-family hardening: route a subset through the D-13 cross-family adversary so within-family shared bias (which same-model resampling structurally misses — the GD-5 limit) is caught (SAC3). HARD BOUNDARY: the done-gate threshold is ELAI-MEASURED on an ELAI gold set (L-10), NEVER a paper AUROC (ID-5). GD-1: do NOT re-implement I-4 resampling or the F-12 NLI judge — call them.

Estimated effort: ~1 session (~M-L). The resample legs invoke the model N times; the deterministic fixture tier ships now, the live multi-sample run is RunPod/cost-gated (W-10) and routed to the free-pool/Batch lane (J-54/H-5).
Pre-requisites (re-grep live per W-7): I-4 run_best_of_n (search/ladder.rs); D-3 VerificationAggregator; D-13 cross-family adversary (CounterEvidenceSchema); D-7 cross-domain canary; F-12 DeterministicNliJudge (live_cli post-reflect tick); L-1 ClaimSet; L-2 PredictorSignal.
Parallel-safe with: L-3, L-5 (disjoint files; all feed L-2). Depends L-2 for the PredictorSignal interface.
Source: cgg-hallucination-causal-system-2026-06-27.md S4 + paper §4.2 predictor 3 + L-CGG-GROUNDING-MAP NN-3/TS-10/TS-11/DC-3. Doctrine: L-0_INDEX.md (HG-2, HG-3, ID-5, GD-5).

## Context
Why this matters: round-trip (L-3) catches invented refs but not confident wrong REASONING that cites real symbols; semantic entropy is the orthogonal signal (a different mechanism, which is exactly what HG-2's >=2-agree rule needs — two distinct predictors, not two views of one). The key subtlety the semantic-entropy literature solved: phrasing variance is NOT disagreement — the same true answer phrased five ways must cluster together; only semantically DISTINCT answers count. That is why L-4 clusters via NLI entailment (F-12 DeterministicNliJudge, local + deterministic, keeps ELAI_SEED), not embedding cosine or an LLM grader. The honest residue (GD-5): same-model resampling cannot catch a belief the model is confidently-wrong-about across all its seeds (a training-data-shared error). The cross-family adversary (D-13, a different pretraining family) is the mitigation — disagreement across families catches within-family bias (SAC3). It is a mitigation, never an elimination; that limit is named in-file.

## Existing state (D/F/I coverage)
- I-4 evidence-ranked search ladder [DARK(ELAI_SEARCH_LADDER_ENABLE), search/ladder.rs] — run_best_of_n over SessionDag; the resample substrate. [MAP]
- D-3 VerificationAggregator [DARK] — >=2-concur; the agreement count L-4 turns into entropy. [MAP]
- D-13 cross-family adversary [DARK(ELAI_ADVERSARY_ENABLE)] — different pretraining family, CounterEvidenceSchema; refuses same-family dispatch. [MAP]
- F-12 semantic contradiction NLI [PARTIAL] — Tier-1 DeterministicNliJudge wired in live_cli post-reflect tick; local deterministic (NOT cloud — keeps ELAI_SEED). The meaning-clusterer. [MAP]
- D-7 cross-domain canary [PARTIAL] — 5 families, block on >5% regress. [MAP]

## Reads (mandatory before coding)
1. _grounding/L-CGG-GROUNDING-MAP.md (NN-3, TS-10/11, DC-3); cgg-hallucination-causal-system-2026-06-27.md S4; paper §4.2 predictor 3; the WV-2 prior art (SelfCheckGPT/semantic-entropy/SAC3 — WebSearch-verify the arXiv IDs first, PENDING-VERIFY).
2. search/ladder.rs (run_best_of_n); the F-12 DeterministicNliJudge; D-13 adversary dispatch; L-1 ClaimSet + L-2 PredictorSignal.
3. F-0 W-1/W-10; G-0 GD-4/GD-5; CLAUDE.md TD-3/ID-5, no-.unwrap, blake3.

## Tasks (WAVE 2 — resample, cluster by meaning, cross-family)
1. the resample driver (~50 LOC): for a Behavior/Causal/Api claim flagged medium by L-3's cheap residual, resample N alternative answers via I-4 run_best_of_n (N from L-2's cost-budget, J-84). NO new sampling — call the ladder.
2. the meaning-cluster + entropy (~70 LOC): cluster the N samples by NLI entailment-equivalence via the F-12 DeterministicNliJudge (paraphrases cluster, distinct meanings split); compute semantic entropy over the clusters. Emit a P3 PredictorSignal{SemanticEntropy, entropy}. NEVER embedding cosine, NEVER an LLM-judge (TD-3).
3. the cross-family arm (~40 LOC): route a subset through the D-13 adversary (different family); a same-family-agreed claim the cross-family contradicts emits Kind CrossFamilyDiverge (the GD-5 mitigation). Honest residue noted in-file: this mitigates, never eliminates, shared-wrong beliefs.
4. wire into L-2's cost-gated predictor call (~20 LOC). default OBSERVE-only (GD-4).
5. workflow/reasoning/L-4-decisions.md.

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/claim/semantic_entropy.rs — resample driver + meaning-cluster + entropy + cross-family arm + P3 PredictorSignal
consumes_symbols:
  - search/ladder.rs::run_best_of_n (I-4 resample — COMPOSE, never re-implement)
  - F-12 DeterministicNliJudge (the meaning clusterer); D-13 cross-family adversary; D-3 agreement
  - claim/mod.rs::ClaimSet (L-1); claim/risk.rs::PredictorSignal (L-2)
reachable_from:
  - claim/risk.rs::score_claim_risk > semantic_entropy (L-2 cost-gate, only on medium+ cheap-risk) [PRODUCTION P3 predictor]
  - a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO embedding-cosine clustering and NO LLM-judge clustering (NLI entailment only, TD-3)
  - NO same-family self-critique presented as cross-family (D-13 refuses same-family — keep it)
  - NO paper AUROC as the done-gate (ID-5 — ELAI-measured at L-10)
  - NO re-implementation of run_best_of_n or the NLI judge (GD-1 P0); NO .unwrap on prod; blake3 not sha2
frontend_pair:
  N/A with justification — feeds L-2's risk tile. Justified.
observable_effect:
  trigger: "resample N=5 on a claim, with one fixture where all 5 share a meaning and one where they split into 3 meanings (via L-2's cost-gate); plus a same-family-agreed claim the cross-family adversary contradicts"
  expect_ssb: "+1 SemanticEntropyMeasured (Pattern-2; Kind: SeedsAgree / SeedsDiverge / CrossFamilyDiverge) OR reuse a verifier row; baseline+0..1 from LIVE ~268/577 (re-grep, W-9)"
  expect_artifact: "the 5-agree case scores low entropy (SeedsAgree); the 3-cluster case scores high entropy (SeedsDiverge); the cross-family-contradicted case flags CrossFamilyDiverge; NEGATIVE: 5 PARAPHRASES of one answer cluster together (low entropy — phrasing variance is not disagreement)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/L-plan/L-4_semantic_entropy_inter_seed_predictor.md --run-trigger
rg "entropy|NliJudge|cluster" .elai_cc/crates/orchestrator/src/claim/semantic_entropy.rs   # P3 present
rg -ci "cosine|embedding.*cluster|llm.*judge" .elai_cc/crates/orchestrator/src/claim/semantic_entropy.rs  # MUST be 0 (NLI only)
test -f workflow/reasoning/L-4-decisions.md
```

## ACCEPTANCE
- [ ] meaning-clustering is NLI-entailment (F-12), never cosine or an LLM-judge (TD-3).
- [ ] paraphrases of one answer cluster together (phrasing variance != disagreement).
- [ ] HG-2: P3 is mechanistically distinct from P1/P2 (decorrelated; verify it does not collapse to round-trip).
- [ ] GD-5: the shared-wrong-belief limit is named in-file; the cross-family arm is the stated mitigation.
- [ ] ID-5: the entropy threshold is ELAI-measured at L-10, never a paper AUROC.
- [ ] GD-1: I-4 + F-12 consumed, not re-implemented.
- [ ] cargo check --workspace GREEN; auditor PASS on -p orchestrator. No new dependency (the 3 WV-2 arXiv IDs are CITATIONS, not deps — WebSearch-verify before citing).

## WIRING DONE-GATE (clause by clause)
- W-1: done = the live cost-gate emits a P3 entropy that flips a claim to High when seeds diverge (not cargo-green).
- W-2: no flag (a predictor; the gate flag is L-7).
- W-3: composes BUILT I-4/F-12, not a summary-only port.
- W-4: frontend_pair N/A (feeds L-2); justified.
- W-5/W-10: the deterministic fixture tier ships now; the live N-sample run is RunPod/free-pool-gated (W-10).
- W-7: search/ladder.rs + F-12 + D-13 re-grepped at edit.
- W-9: SSB prefer-reuse or +1 from LIVE baseline.
- GD-2: the entropy is recomputable from the recorded cluster assignments under ELAI_SEED.
- GD-5: shared-wrong-belief limit named; cross-family is mitigation not elimination.

## Tests to add
- crates/contracts/tests/l4_semantic_entropy_contract.rs (~90 LOC): (a) N samples sharing a meaning > low entropy; (b) N splitting into k>1 meanings > high entropy; (c) paraphrases of ONE answer cluster together (not flagged); (d) clustering uses NLI not cosine (assert no embedding path); (e) the cross-family arm flags a same-family-agreed-but-cross-family-contradicted claim; (f) the entropy is recomputable from the cluster assignments (GD-2); (g) D-13 refuses a same-family dispatch.

## Files touched
- CREATE: .elai_cc/crates/orchestrator/src/claim/semantic_entropy.rs (~160 LOC)
- MODIFY: claim/risk.rs (~20 LOC: wire P3 into the cost-gate)
- CREATE: crates/contracts/tests/l4_semantic_entropy_contract.rs (~90 LOC)
- CREATE: workflow/reasoning/L-4-decisions.md

## Citations / Grounding
- arXiv:2606.27326 §4.2 predictor 3 (inter-seed variance) [EXTERNAL/DESIGN-RATIONALE]; SelfCheckGPT / Semantic Entropy (Nature) / SAC3 (WV-2, PENDING-VERIFY the arXiv IDs before they ship in code); cgg-hallucination-causal-system-2026-06-27.md S4.
- ELAI ground truth (L-CGG-GROUNDING-MAP NN-3/TS-10/11/DC-3): search/ladder.rs run_best_of_n, F-12 DeterministicNliJudge, D-13 adversary (re-grepped at edit, W-7).
- CLAUDE.md: TD-3, ID-5 (ELAI-measured threshold), no .unwrap, blake3-only; GD-5 honest residue.

## Open questions / risks
- N (sample count) vs cost: N is the dominant cost lever; bound it via L-2's J-84 budget, run on the free-pool/Batch lane (J-54/H-5), and OBSERVE-first to find the N where the entropy signal stabilizes (ID-5). Decide the N schedule at L-10.
- Decorrelation from L-3: if semantic entropy and round-trip residue correlate >threshold (H-40), HG-2's "two distinct predictors" becomes fake convergence. Measure their correlation at L-10 and, if collapsed, treat them as ONE signal for the >=2-agree rule.

## Next
L-4 is the orthogonal uncertainty signal that makes HG-2's convergence rule real. With L-3 (round-trip) + L-4 (semantic entropy) + L-5 (do-test), L-2 has three mechanistically-distinct predictors to fuse.

End of L-4.
