# N-4: one Claim/Belief object + receipt memo — one canonical claim type, receipts memoized as case nodes

- Track: TIER 2 (HIGH) | lever: cheaper-reasoning-on-your-repo | finding: W-2+W-4+W-22+W-36 | grounding: STRONG | nature: BUILD-on-shipped | effort: L | WAVE 3

- Premise: four independently-designed typed objects all mean "a statement the system may or may not trust, with evidence attached," each with its own validators, SSB rows, staleness story. And nothing lets a RECURRING claim reuse its prior receipt — every session re-pays the full checker/resample cost even when the identical claim (same symbol, same commit-window) was proven yesterday. N-4 unifies the type AND adds the memo, doubling as the cost-amortizer of both the reasoning engine and the CGG.

## Steal/build
ONE canonical Claim/Belief object: L-1 ClaimSet{ClaimKind,span,refs} ARE J-83 BeliefState entries{claim,receipt,abstain,DKG-provenance} (L-8 already threads this — finish it); the receipt attached at the J-83 gate IS the evidence L-3 round-trips; K-12 critical_atoms must_preserve = has-receipt; GRC grounding tags "type:reference" join the one object; J-21 re-derives the SAME object against commit+AST.
AND the RECEIPT MEMO (W-4+W-22+W-36): before running an expensive checker/resample (D-11 registry / F-4 tests / L-5 do-tests / L-4 resampling N=5), look up claim-identity/trigger_signature in the J-96 case graph — a HIT re-validates cheaply via J-21 commit+AST identity (already specified for memory) instead of re-running the intervention; a MISS mints a new node. This generalizes C-30's OracleAnswerCache (query_hash,project_hash)->answer,trust-TTL into the receipt memo, invalidated by the F-5 staleness cascade. Consulted by BOTH the J-83 reasoning receipt-gate AND the L-2 CGG first (W-36). Biggest win on the checker-poor local 7B lane; "gets cheaper on YOUR repo" (K-18 moat #2) extended to REASONING.
EXCLUDED: re-authoring the C-30 cache (generalize it); re-designing L-8's carrier (it exists — align K-12 + GRC tags to it); a fifth claim type.

## Existing ground truth (PLAN-asserted; executor re-greps per W-7)
- L-1 ClaimSet, J-83 BeliefState entries, K-12 critical_atoms, GRC "type:reference" tags all exist as distinct typed objects (feature-map claim group; L-8 threads L-1 over J-83).
- J-96 mints case nodes on F-4 proof with trigger_signature + blast cone + proof receipt; J-97 is demonstration-injection memory (not yet a receipt cache).
- C-30 OracleAnswerCache memoizes (query_hash,project_hash)->answer with trust-tiered TTLs — the exact memo shape.
- J-21 memory validation re-derives against commit+AST; F-5 staleness cascade exists.
- I-8 dead-end AST hashing already caches "what FAILED" (the negative-space twin) — N-4 adds "what was PROVEN".

## REVISED scope (W-10 tiered)
- SHIP NOW (Tier-1): align K-12 + GRC tags onto the L-8/J-83 carrier (the schema-alignment residual, W-2 is half-done) + the memo lookup-before-checker at the J-83 gate, re-validated via J-21, invalidated via F-5.
- DEFER (Tier-2): the L-2 CGG consulting the same memo (W-36) — lands right after the J-83 side proves the identity+invalidation is sound.

## Integration
- crates/orchestrator ssb::payloads BeliefState + the J-83 gate + L-2 CGG call sites; the C-30 cache generalization.
- frontend_pair: the existing belief/claim drawer gains a memo-hit indicator, OR N/A justified.

## WIRING_CONTRACT
produces_symbols:
  - the aligned canonical Claim/Belief object (K-12 + GRC tags folded onto the L-8/J-83 carrier)
  - the receipt-memo lookup (generalized C-30 cache, keyed by claim-identity/trigger_signature)
consumes_symbols:
  - L-1 ClaimSet / J-83 BeliefState (L-8 carrier) / K-12 critical_atoms / GRC tags
  - J-96 case nodes / C-30 OracleAnswerCache / J-21 commit+AST identity / F-5 staleness
reachable_from:
  - the J-83 reasoning receipt-gate on a live task  [PRODUCTION reasoning seam]
  - the L-2 CGG predictor path (Tier-2)  [PRODUCTION hallucination-gate seam]
stub_ban:
  - NO fifth claim type
  - NO re-implementation of the C-30 cache
  - NO memo Hit returned on F-5-invalidated (changed) code
  - NO .unwrap()/.expect() on prod paths; blake3 for claim-identity hashing
frontend_pair: memo-hit indicator on the belief/claim drawer OR "N/A justified"
observable_effect:
  trigger: "assert a recurring claim about UNCHANGED code twice on a live task"
  expect_ssb: "first: checker runs + a J-96 node minted; second: +receipt_memo(Hit) and the checker is skipped"
  expect_artifact: "the second pass is measurably cheaper (no checker call); NEGATIVE: a claim on CHANGED (F-5-invalidated) code returning a stale Hit must FAIL"

## Grounding
- interlace doc W-2 (four claim types), W-4 (receipt economy has no memo), W-22 (C-30 is the precedent), W-36 (CGG wants the memo day one), TIER-2 item 4; L downgrade note (L-8 half-plans W-2).
- memory project_b40_shipped (idea_framing types), reference material on J-83/L-1/K-12.

## COMPOSES-WITH
L-1, J-83 BeliefState (L-8), K-12, GRC tags, J-96, C-30, J-21, F-5, L-2..L-6. Re-implements NONE (GD-1). Makes the hallucination gate (L) automatically cover reasoning (J-83) and handoffs (K-12) instead of each re-implementing "is this statement grounded," and makes reasoning cheaper on the weakest lane over time.

End of N-4.
