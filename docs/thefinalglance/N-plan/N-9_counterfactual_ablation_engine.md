# N-9: one counterfactual / ablation engine — one drop-one/inject-one engine over the journal, target as parameter

- Track: TIER 3 (MEDIUM-HIGH) | lever: dedupe-the-subtlest-measurement | finding: W-20+W-27 | grounding: STRONG | nature: BUILD | effort: M | WAVE 6 | depends: N-1

- Premise: the SAME mechanism — re-run/re-decide with one factor removed or injected, score the delta off journal outcomes — is specified EIGHT times pointed at different targets. Four attribution engines + four ablation harnesses, each independently specified, each a hard-to-get-right implementation of the subtlest measurement in the system.

## Steal/build
ONE counterfactual/ablation engine over the SSB with TARGET as a parameter (role / verifier / ranker / rule / memory-entry / case), plus eight THIN adapters (ND-5):
- attribution side: C-45 CANTANTE per-role credit (drop-a-role) / I-28 DAG credit (winner vs highest-vetoed sibling) / J-105 per-memory-entry causal ROI (counterfactual replay) / J-109 AD-4 pre-write counterfactual replay;
- ablation side: E-6 iso-budget role audit (delete-a-role unless >1-sigma lift) / E-10 critic ablations / H-40 verifier retention by decorrelation / J-75 RRF ablation (drop-a-ranker).
Natural consumer of N-1's outcome labels (W-11) — the delta is scored against the same ground-truth labels.
EXCLUDED: re-implementing the replay mechanism eight times; a ninth attribution target; an adapter carrying business logic (ND-5 — the core does the replay+scoring, the adapter only names the target).

## Existing ground truth (PLAN-asserted; executor re-greps per W-7)
- C-45/I-28/J-105/J-109 (attribution) + E-6/E-10/H-40/J-75 (ablation) all exist as independent PLANNED units (feature-map attribution + ablation groups).
- all are drop-one/inject-one over the same journal outcomes; K-18 reuse-ON/OFF arms are the same shape.
- N-1 supplies the outcome labels; N-3's deflation applies when the ablation feeds a promotion.

## REVISED scope (W-10 tiered)
- SHIP NOW (Tier-1): the engine core (drop-one/inject-one replay + delta-scoring over the SSB) + 3-4 adapters (role, verifier, ranker) proving the parameterization.
- DEFER (Tier-2): the memory/case/DAG-credit adapters (land as those subsystems stabilize; the core is shipped and proven first).

## Integration
- crates/orchestrator counterfactual engine + the eight call sites (as adapters).
- frontend_pair: reuse the existing attribution drawers, OR N/A justified.

## WIRING_CONTRACT
produces_symbols:
  - the counterfactual/ablation engine core (target as parameter) + the thin adapters
consumes_symbols:
  - C-45 / I-28 / J-105 / J-109 (attribution) + E-6 / E-10 / H-40 / J-75 (ablation) as adapters
  - N-1 outcome labels; the SSB event stream
reachable_from:
  - a live role/verifier/ranker ablation through the ONE engine  [PRODUCTION ablation seam]
  - a #[cfg(test)] caller NEVER satisfies this (R7)
stub_ban:
  - NO eight separate replay implementations
  - NO adapter carrying business logic (ND-5 — delegate to the core)
  - NO delta scored off anything but N-1 labels / the journal (GD-2)
  - NO .unwrap()/.expect() on prod paths
frontend_pair: existing attribution drawers OR "N/A justified"
observable_effect:
  trigger: "a live role (or verifier, or ranker) ablation through the one engine"
  expect_ssb: "+counterfactual_scored(Role|Verifier|Ranker|Memory|Case)"
  expect_artifact: "the delta is computed by the core from journal labels; NEGATIVE: an adapter that carries business logic instead of delegating to the core fails ND-5"

## Grounding
- interlace doc W-20 (four counterfactual engines), W-27 (four ablation harnesses), TIER-3 item 9.
- N-1 (the labels), N-3 (deflation when it feeds promotion).

## COMPOSES-WITH
C-45/I-28/J-105/J-109, E-6/E-10/H-40/J-75, N-1. Re-implements NONE (GD-1). Dedupes eight hard-to-get-right implementations of the subtlest measurement into one core + thin adapters.

End of N-9.
