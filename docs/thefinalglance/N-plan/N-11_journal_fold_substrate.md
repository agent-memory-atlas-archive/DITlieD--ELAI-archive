# N-11: journal-fold substrate — extend L-16's pure fold beyond context to five re-derive surfaces, one Kani proof

- Track: TIER 3 (MEDIUM-HIGH) | lever: one-proof-five-surfaces | finding: W-10 | grounding: STRONG | nature: BUILD-on-shipped | effort: M-L | WAVE 7

- Premise: L-16's pure fold(log,budget,policy)->context-view is the right unifier; five OTHER surfaces re-derive-from-journal ad hoc, each needing its own GD-2 audit. One fold core with per-surface fold functions as data/plugins means ONE J-68 Kani proof covers all five — instead of five ad-hoc "re-derive byte-identically" implementations. Kills a whole drift class.

## Steal/build
Extend L-16's pure fold core beyond context: per-surface fold FUNCTIONS as data/plugins for the five re-derive surfaces — canvas altitudes (J-31/J-60/J-98/K-23), I-39 plan status, K-24 billing balance, M-3 fleet rollup, K-22 coverage. ONE J-68 Kani correctness proof on the fold core then covers all five. Ships the ND-2 RETIREMENT map for the paths L-16 leaves live: recover_stubs end-of-life; declare H-49 CAS/undo-tree THE log representation (one store).
This is a BUILD-on-shipped: L-16 already replaces compact_session, keeps the mutate path one release as rollback, and re-expresses E-25/H-6/3E/H-50 as fold STRATEGIES; L-18 collapses K-12+H-49 into projection corollaries. N-11's residual is ONLY (a) the five non-context surfaces adopting the fold core, (b) the one Kani proof, (c) the recover_stubs/H-49 retirement. Do NOT re-author the fold (GD-1).
EXCLUDED: re-authoring L-16's fold; a sixth context core (the whole point is to avoid the n+1 standards failure); retiring a path without user confirmation (ND-2).

## Existing ground truth (PLAN-asserted; executor re-greps per W-7)
- L-16 pure fold(log,budget,policy)->context-view exists; CC-1..CC-4 collapse 4 context mechanisms into fold() parameters; L-18 collapses losslessness+time-travel into it (L-plan).
- J-31/J-60/J-98/K-23 canvas altitudes "re-derive byte-identically from the journal"; I-39 status, K-24 balance, M-3 rollup, K-22 coverage each recomputable from journals (feature-map fold group).
- J-68 Kani is planned for the fold core; H-49 CAS/undo-tree is the log-representation candidate.
- recover_stubs marked "legacy-only" needs an explicit end-of-life (interlace doc W-13 residual).

## REVISED scope (W-10 tiered)
- SHIP NOW (Tier-1): the fold core adopted by 2-3 of the five surfaces (canvas + I-39 status) + the J-68 Kani proof on the core + the recover_stubs/H-49 retirement map (user-confirmed).
- DEFER (Tier-2): K-24 balance + M-3 rollup + K-22 coverage adopting the fold (land with those units).

## Integration
- crates/orchestrator fold core (extend L-16) + the five surface fold-fns; H-49 CAS as the log store.
- frontend_pair: canvas altitudes already consume folds — verify byte-identity, do not re-wire.

## WIRING_CONTRACT
produces_symbols:
  - the extended fold core (L-16 + per-surface fold-fns as data) + the J-68 Kani proof on the core
  - the ND-2 retirement map (recover_stubs EOL; H-49 CAS as THE log representation)
consumes_symbols:
  - L-16 fold (EXTEND, do not re-author), J-68 Kani, H-49 CAS/undo-tree
  - the five re-derive surfaces (canvas / I-39 / K-24 / M-3 / K-22)
reachable_from:
  - two of the five surfaces re-derived through the ONE fold core on a live session  [PRODUCTION fold seam]
  - a #[cfg(test)] caller NEVER satisfies this (R7)
stub_ban:
  - NO re-authoring of L-16's fold
  - NO sixth context core
  - NO surface still re-deriving off its own ad-hoc path (conformance FAIL)
  - NO path retirement without user confirmation (ND-2)
frontend_pair: canvas altitudes consume the fold (verify byte-identity)
observable_effect:
  trigger: "re-derive two of the five surfaces through the one fold core with a pinned ELAI_SEED"
  expect_ssb: "reuse L-16 fold events; no new type"
  expect_artifact: "byte-identical re-derivation across the two surfaces + the retirement switch removes the old path; NEGATIVE: a surface still re-deriving off its own ad-hoc code path must FAIL the conformance check"

## Grounding
- interlace doc W-10 (projection/fold engines re-implemented per surface) + its L downgrade note (L-16 ~specifies retirement; residual = recover_stubs EOL + H-49 as the store), TIER-3 item 11.
- memory project_hplan_track8_authored (H-49 CAS/undo-tree); L-16/L-18 (the fold unifier).

## COMPOSES-WITH
L-16 (extend), J-68 Kani, H-49 CAS, the five re-derive surfaces. Re-implements NONE (GD-1). One fold core, one proof, five surfaces — prevents the unifier from becoming a sixth layer.

End of N-11.
