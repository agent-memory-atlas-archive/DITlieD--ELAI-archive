# N-3: generalized anti-selection-bias promotion primitives — lift H-17/H-18/H-19 from finance to every promotion loop

- Track: TIER 2 (HIGH) | lever: gated-against-its-own-search | finding: W-30 | grounding: STRONG | nature: ACTIVATION | effort: M-L | WAVE 3 | depends: N-1

- Premise: H-17 trial ledger + H-18 deflated-significance + H-19 reusable-holdout were built for the TRADING loop only, but every promotion loop in the corpus runs the same max-of-N-noisy-scores search and is upward-biased everywhere. A rule/prompt/case that "won" after N tries is upward-biased the same way an in-sample Sharpe>1.0 after ~7 noise trials is. This is the single most valuable UNPLANNED generalization: it hardens ~8 loops with machinery already fully designed for one.

## Steal/build
Lift the three finance primitives into DOMAIN-GENERIC promotion primitives keyed by (candidate-family) instead of (finance-candidate), read via the ID-6 ledger hook:
- H-17 persisted, agent-unresettable per-(candidate-family) trial ledger;
- H-18 deflation/threshold correction (DSR/PBO/MinBTL, N from the ledger, fail-closed on missing N) applied at every promotion gate;
- H-19 reusable-holdout DP query budget (Thresholdout) for every reusable eval corpus (J-56/L-20/D-6).
The loops it hardens: 3F rule SHADOW->ACTIVE, J-57 prompt promotion, J-97 case-reuse fitness, B-23/H-45 configs, I-4/I-25 candidate ladders, L-11 OQ thresholds. (I-30/I-31 already route promotions "through H-17 trial-N + H-18 deflation" — proof the generalization is natural; N-3 extends it to the non-Track-17 loops.)
EXCLUDED: re-implementing DSR/PBO/MinBTL (re-KEY the shipped H-18 impl); any new promotion loop (N-3 hardens existing ones); a promotion that runs without the ledger (fail-closed on missing N, per H-18).

## Existing ground truth (PLAN-asserted; executor re-greps per W-7)
- H-17/H-18/H-19 shipped/planned FINANCE-ONLY (TB-1); only H-47 partially reuses H-19 (H-plan Track 7).
- ID-6 = "learned artifacts promote through the ledger"; I-30/I-31 already route promotions through H-17+H-18.
- the ~8 promotion loops each promote on max-of-N without trial-count deflation (feature-map promotion group).
- N-1 supplies the outcome labels the deflation corrects against (the ground truth: did the promoted artifact actually help, per the journal).

## REVISED scope (W-10 tiered)
- SHIP NOW (Tier-1): re-key the H-17 ledger + H-18 deflation to (candidate-family) and wire the 3F rule + J-57 prompt + J-97 case gates to read it. Fail-closed on missing N.
- DEFER (Tier-2): the full Thresholdout DP budget on every corpus (start with the ledger+deflation; the H-19 holdout budget is the second wave, needs N-1's corpus stable first).

## Integration
- crates/orchestrator promotion sites (rules lifecycle, prompt/case/config archives); the ID-6 ledger hook.
- frontend_pair: a deflation column on the existing promotion drawers, OR N/A justified.

## WIRING_CONTRACT
produces_symbols:
  - the (candidate-family)-keyed trial ledger + deflation reader (re-keyed H-17/H-18)
  - the promotion-gate wiring at the ~8 sites
consumes_symbols:
  - H-17 trial ledger / H-18 DSR-PBO-MinBTL / H-19 Thresholdout (re-key, do NOT re-author)
  - ID-6 ledger hook; N-1 outcome labels
reachable_from:
  - a real SHADOW->ACTIVE promotion transition (rules/prompt/case gate)  [PRODUCTION promotion seam]
  - a #[cfg(test)] caller NEVER satisfies this (R7)
stub_ban:
  - NO promotion that flips ACTIVE ignoring trial count
  - NO re-implementation of DSR/PBO/MinBTL
  - NO silent pass on missing N (fail-closed)
  - NO .unwrap()/.expect() on prod paths
frontend_pair: deflation column on promotion drawers OR "N/A justified"
observable_effect:
  trigger: "a real SHADOW->ACTIVE promotion with N below threshold"
  expect_ssb: "+promotion_deflated(TrialCountApplied|DeflationBlocked|HoldoutBudgetSpent)"
  expect_artifact: "the promotion is HELD; NEGATIVE: a promotion that ignores trial count and flips ACTIVE must FAIL the gate"

## Grounding
- interlace doc W-30 (the finance-only machinery + the ~8 loops running the same search), TIER-2 item 3; NOTE W-30 landing site (ID-6).
- H-plan Track 7 / Track 17 (H-17/H-18/H-19 + H-47 reuse); memory project_hplan_track7_authored.

## COMPOSES-WITH
H-17/H-18/H-19 (re-keyed), ID-6, N-1 labels, the ~8 promotion loops (as consumers). Re-implements NONE (GD-1). Turns "our self-improvement is gated" into "gated against the statistics of its own search."

End of N-3.
