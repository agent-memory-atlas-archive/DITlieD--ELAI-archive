# J-20: self-improvement non-stationary bandit + mandatory security-invariant promotion gate

- Track J (pillar-SOTA recheck extensions) | lever PROVABLE | grounding STRONG (DS-TS / TS-CD / Misevolve / When-Does-Verification all VERIFIED) | nature BUILD (new file SUPERSEDING the plain-TS slices of D-5/D-29/H-48 — never amends them) | effort M | WAVE 3 | depends H-43/H-44 fitness LIVE for the reward, I-24 isomorphic canary

- Premise: the self-improvement loop's execution-grounded fitness is the correct keystone (verify-p6 KEEP, anti-collapse-validated), but the PROMOTION mechanism has two grounded weaknesses. (1) Plain Thompson sampling is misspecified for the NON-STATIONARY task stream a self-improving agent generates — a config good-for-Python promoted on a Python-heavy window underperforms on Rust and may not trip the auto-revert if the aggregate regression stays <10% while a subdomain is severe. (2) There is no SECURITY-INVARIANT gate on promotion — security degrades across evolutionary pathways regardless of task fitness (Misevolve: a coding agent's refusal rate fell 99.4->54.4% and ASR rose 0.6->20.6% AFTER benign memory evolution, with post-hoc corrections failing to restore alignment). The cross-family secondary verifier the report also recommends is ALREADY ours (VD-4 / H-45 / D-13) — J-20 references it, does not rebuild it.

- NEVER-AMEND NOTE (user rule 2026-06-12): D-5/D-29 (BAI Thompson) and H-48 (BAI bandit on real reward) are authored. Per the user's standing rule, old plan files are not re-checked or re-implemented, so J-20 is a NEW file that SUPERSEDES their plain-TS promotion mechanism; the executing agent walks J-20, not an amendment block inside D-5/H-48.

## Steal/build
1. NON-STATIONARY BANDIT: replace plain Thompson sampling in the promotion path with a non-stationary variant — Discounted TS (DS-TS, down-weights old rewards) or Sliding-Window TS, plus a change-point detector that RESETS the posterior on a detected task-distribution shift (TS-CD). The reward is still the execution-grounded fitness (H-43); the change is HOW the posterior tracks it under shift. This catches the good-for-Python-bad-for-Rust failure: a subdomain regression triggers a change-point reset, not a silent aggregate-masked promotion.
2. SECURITY-INVARIANT SUITE AS A HARD PROMOTION GATE: every candidate config must pass a CWE/SAST-grounded security-invariant suite BEFORE promotion — not an optional audit, a blocking gate. The suite is the SAME class as VibeGate F-2's CWE rules + F-10's per-iteration security-monotonicity delta, applied at the promotion boundary: a candidate that raises any tracked CWE class vs the parent = BLOCKED regardless of fitness gain. The threshold is ELAI-CALIBRATED to our own baseline (the IEEE "37.6% after 5 iterations" figure is WEAK / unverified — used as a direction, the gate's trip threshold is measured on ELAI's own CWE classifier, never the literal).
3. cross-family secondary verifier on promotion = CONFIRMATION of the existing VD-4 / H-45 / D-13 mechanism (When-Does-Verification: cross-family > self-verification across 37 models / 9 benchmarks, post-training strengthens cross-family). J-20 WIRES the existing cross-family adversary into the promotion gate (the verifier already exists; the wiring asserts it runs on every promotion), does not re-author it.
EXCLUDED: re-authoring the BAI/Thompson substrate of D-5/D-29/H-48 (J-20 supersedes the MECHANISM, reuses the bandit scaffolding); a NEW cross-family verifier (D-13 exists); the 37.6% literal as a threshold; any LLM as the promotion gate (TD-3: the security suite + execution fitness + cross-family verifier are deterministic/measured; the LLM is advisory only).

## Existing ground truth (grep-confirmed; re-grep at edit, W-7/W-9)
- SELF-MODIFICATION CORE [WIRED] L1/L2/L3, DGM sigmoid parent-select, AllowedConfigKey closed enum, 4 safety layers, auto-revert >10% over 3 (feature map 1.7; B-23, phase-4H) — the promotion path J-20 hardens.
- BAI THOMPSON PROMOTION [DARK] BaiPromotionDecision SSB, D-29 hierarchical non-IID hardcoded FamilyId(0) (feature map 1.7; D-5/D-29) — the plain-TS mechanism J-20 supersedes; D-29's non-IID intent is exactly what DS-TS/TS-CD delivers properly.
- DGM VERIFICATION FILTERS [PARTIAL] H-43 execution-grounded fitness (FitnessEvaluated 221), H-44 trap detection (222), H-45 promotion scoring (223), H-48 BAI bandit on real reward (feature map 1.7) — J-20's reward IS H-43's fitness; J-20 wires H-48's bandit as non-stationary.
- ADVERSARY ROLE CROSS-FAMILY [DARK] D-13 + H-45 + VD-4 (different family from executor) — the cross-family verifier J-20 wires into the promotion gate.
- VibeGate F-2 CWE ruleset (gate_recall=0.6875) + F-10 per-iteration security-monotonicity (CWE-class delta) — the security-invariant suite J-20 applies at promotion.
- DARWINIAN LOOP [DARK] ELAI_DARWINIAN_LOOP_ENABLE — the whole loop stays behind the kill-switch; J-20's gate is part of what makes promotion safe enough to eventually un-dark.

## Integration
- crates/orchestrator self-mod promotion path: swap plain TS for DS-TS/SW-TS + change-point reset (~120 LOC over the existing bandit scaffolding); wire the F-2/F-10 security suite as a blocking pre-promotion check (~80 LOC); assert the D-13 cross-family verifier runs on every promotion (~40 LOC wiring).
- +SecurityInvariantGate SSB (Pattern-2 single-variant; Kind: NonStationaryReset / SecurityInvariantChecked / CrossFamilyVerified / PromotionBlocked). baseline+1 from LIVE 231/542 (grep 2026-06-12); re-read at edit (W-9). PREFER-REUSE: extend H-45 PromotionScoreAggregated / H-48 bandit rows with a security-verdict + change-point field if cleaner (net 0); decide at edit.
- frontend_pair: the security-gate verdict + change-point markers on the existing ProposalsPanel / MsiPanel (B-23) or MutationArchivePanel (D-26) surface; cross_lang_diff if surfaced (W-4).

## WIRING_CONTRACT
produces_symbols:
- crates/orchestrator self-mod promotion: non-stationary TS (discount/sliding-window) + change-point posterior reset (superseding plain TS)
- the security-invariant pre-promotion gate (F-2/F-10 CWE suite as a BLOCKING check, not an audit)
- the cross-family-verifier-on-every-promotion assertion (wiring D-13, not re-authoring)
consumes_symbols:
- H-43 execution-grounded fitness (the reward signal; NOT reauthored)
- D-13 / VD-4 cross-family adversary (the verifier; NOT reauthored — wired into the gate)
- F-2 / F-10 CWE classifier (the security suite; NOT reauthored)
- the D-5/D-29/H-48 bandit scaffolding (superseded mechanism, reused scaffolding)
reachable_from:
- a real self-mod promotion decision on an `elai` run with ELAI_DARWINIAN_LOOP_ENABLE (the production promotion path)
- a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
- NO plain stationary Thompson sampling in the promotion path (the non-stationarity is the point)
- NO promotion without the security-invariant suite passing (blocking gate, not optional audit)
- NO same-family / self-critique as the promotion verifier (D-13 cross-family; VD-4 stronger-than-generator; banned per ledger)
- NO LLM as the promotion gate (TD-3; the gate is execution-fitness + security-suite + cross-family verifier)
- NO default-ON promotion until FP/over-block bounded on >=3 real runs (GD-4) + the loop stays behind ELAI_DARWINIAN_LOOP_ENABLE with a wire-dark-allowlist row (W-2/R9)
- NO using the 37.6% literal as the security trip threshold (ELAI-calibrated; ID-5)
frontend_pair: security-gate + change-point on ProposalsPanel/MsiPanel/MutationArchivePanel (decide at edit) or W-6-justified absence.
observable_effect:
- trigger: "a self-mod candidate config promotion under a SHIFTED task stream (fixture: a config that scores high on a Python-heavy window then faces a Rust window), with ELAI_DARWINIAN_LOOP_ENABLE on"
- expect_ssb: "NonStationaryReset on the detected distribution shift (the posterior resets rather than carrying stale Python reward); SecurityInvariantChecked on the candidate; CrossFamilyVerified by a different-family adversary"
- expect_artifact: "a candidate that raises a CWE class vs the parent is PromotionBlocked despite a fitness gain; a subdomain (Rust) regression triggers the change-point reset rather than an aggregate-masked promotion. NEGATIVE (R2): a candidate evaluated with plain stationary TS (no reset) FAILS the contract test; a promotion that skipped the security suite FAILS; a same-family verifier on the promotion FAILS the cross-family assertion; the security suite pointed at a candidate it cannot classify FAILS-LOUD, never passes."

## Grounding
- DS-TS arXiv:2305.10718 (VERIFIED STRONG, "Discounted Thompson Sampling for Non-Stationary Bandit Problems", O~(sqrt(T B_T)) regret). TS-CD arXiv:2009.02791 (VERIFIED STRONG, change-detection + posterior reset). SW-TS (JAIR) NOT-FOUND via public search (WEAK; the DS-TS/TS-CD legs carry the grounding, SW-TS is an interchangeable alternative if the JAIR cite surfaces).
- Misevolve arXiv:2509.26354 (VERIFIED STRONG, refusal 99.4->54.4% / ASR 0.6->20.6% post memory-evolution, post-hoc corrections fail) — the security-invariant-gate justification.
- When-Does-Verification arXiv:2512.02304 (VERIFIED STRONG, cross-family > self across 37 models/9 benchmarks) — CONFIRMS the existing D-13/VD-4 mechanism; SATLUTION arXiv:2509.07367 (VERIFIED, proof-checking-easier-than-proof-finding = a genuine stronger-than-generator verifier).
- IEEE ISTAS "37.6% after 5 iterations" NOT-FOUND via public search (WEAK; used as direction only, threshold ELAI-calibrated). Self-Evolution Trilemma arXiv:2602.09877 + Two-Gate arXiv:2510.04399 (both VERIFIED) — ground the immune-tier capacity-cap doctrine (already TD-1..TD-5 / VD-1..VD-5).
- live SSB 231/542 (grep 2026-06-12).

## COMPOSES-WITH
H-43 fitness (the reward), D-13/VD-4/H-45 cross-family verifier (wired into the gate), F-2/F-10 CWE suite (the security gate), D-5/D-29/H-48 bandit (superseded mechanism + reused scaffolding), I-24 isomorphic canary (the anti-gaming backstop on the fitness the bandit optimizes), I-31 GRPO flywheel (downstream; a safe promotion gate is its precondition). Restates none.

## Open questions / risks
- change-point sensitivity: too sensitive resets the posterior on noise (slow convergence); too dull misses the shift (the failure it exists to catch). The discount factor / window size + detection threshold are ELAI-calibrated on the real task stream, bounded over >=3 runs (GD-4).
- security-suite coverage: the F-2 CWE ruleset has gate_recall=0.6875 — the gate is necessary-not-sufficient (R5); a CWE class outside the ruleset's coverage is a false-negative. Declare coverage; the cross-family verifier + execution fitness are the complementary backstops.
- the loop stays dark: J-20 makes promotion SAFER, it does not un-dark the Darwinian loop — that remains a separate GD-4-bounded, user-gated decision behind ELAI_DARWINIAN_LOOP_ENABLE.

## Next
A non-stationary, security-gated, cross-family-verified promotion is the precondition the I-31 GRPO flywheel's BLOCKING GUARDS require. J-20 is the safety upgrade that makes eventually un-darking the loop defensible.

End of J-20.
