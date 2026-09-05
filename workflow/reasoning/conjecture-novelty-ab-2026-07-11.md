# Conjecture-engine NOVELTY A/B — K=10 (2026-07-11)

The creativity leg the block-rate pilot (conjecture-bench-run-2026-07-11.md) never measured.
Harness: .claude/tools/conjecture_novelty_ab.py (seeded, call-cached, resumable). Engine candidate
sets reused from the pilot's saved artifacts; freestyle-10 sets + naive picks regenerated (the
pilot did not persist them). Same generator (kimi-k2-7), same 10 triggers. ~120 free Devin calls.

## VERDICT (plain)

NEGATIVE / NULL on the novelty axis, and it is informative. The engine's structural diversity
machinery (stance rotation + empty-bin forcing) did NOT produce a measurably more diverse
candidate set than one "propose 10 DISTINCT ideas" freestyle call, and the engine's PICKS LOSE
blind pairwise novelty AND judge-bet value against naive best-of-10 picks. Combined with the
pilot, the composite picture is: the engine's proven value is WORLD-COUPLING (real anchors,
3.5x), not idea diversity and not judge-visible pick quality. SF-4: this null ships as-is.

## NUMBERS

M1 lexical set diversity (deterministic 3-gram pairwise distance):
  freestyle more diverse on 10/10 triggers; delta -0.018, CI95 [-0.031, -0.009].
  BUT both arms are near-saturated (0.93-1.00) and the metric carries a CONFOUND: engine
  candidates were normalized to "<effect> (via <family>, targeting <area>)" one-liners, whose
  shared scaffold + repeating family vocabulary mechanically depress n-gram distance. Direction
  unreliable; the honest reading is "both sets lexically distinct, no engine win".

M2 judge mechanism-cluster count (glm-5-2, 2 seeded orderings per set):
  delta -0.7 mean clusters, CI95 [-1.8, +0.25] straddles 0; sign 3 engine / 4 freestyle / 3 tied.
  INCONCLUSIVE, slight freestyle lean. Orderings routinely disagree by 1-2 clusters (instrument
  noise, reported not hidden).

M3 cross-set mechanism uniqueness (judge counts mechanisms one set has that the other lacks):
  engine 6.5 vs freestyle 7.0 mean unique-of-10 — SYMMETRIC. The engine reaches different
  territory but not MORE unique territory than freestyle reaches against it.

M4 blind pairwise PICK judgement (panel glm-5-2 + kimi-k2-7, 2 order-swaps, 40 votes):
  novelty: freestyle 27 - engine 13. value-bet: freestyle 31 - engine 8 - tie 1.
  Judge-robust: glm (cross-family) 15-5 novelty / 15-4 value; kimi 12-8 / 16-4.

## WHY THE PICKS LOSE (three mechanisms, two of them designed-in or instrumental)

1. RECEIPTS-BIASED SELECTION (designed-in, predicted before this run): the engine's select stage
   prefers executed-leg/receipt-backed candidates, so its winners are systematically the most
   CHECKABLE-NOW ideas, not the boldest. The novelty loss at the pick stage is the engine working
   as built. If novelty is wanted in the pick, it must be a separate scored axis, never inferred.
2. JUDGE BLINDNESS TO GROUNDEDNESS (instrumental): a no-file-access judge cannot tell a real
   anchor from a hallucinated one — and the pilot showed 8/10 freestyle picks name FAKE seams.
   The engine's entire proven advantage is INVISIBLE to M4's value-bet, while its costs
   (conservatism, terseness) are fully visible. M4 measures judge-visible plausibility, which is
   freestyle's home turf.
3. PRESENTATION CONFOUND: engine picks are receipts-stripped card skeletons (~220-450 chars);
   freestyle picks are fluent 3-6 sentence restatements. Verbosity was instruction-penalized but
   LLM fluency bias is only partly suppressible.

## SCOPE LIMITS (do not over-read the null)

- Single-shot regime: this measures INSTANTANEOUS set diversity from fresh context. DeepReason's
  measured rotation win (F3) is about fighting repertoire FADE over many cycles on one problem —
  a different regime this bench does not touch. The null here does not refute rotation-under-fade.
- One generator (kimi-k2-7), K=10, pilot-grade CIs. The "10 DISTINCT ideas" single-call baseline
  is strong precisely because one call lets the model self-coordinate distinctness; multi-call
  regimes (where the engine's bin state persists and freestyle has no memory) are untested.
- M2/M3 depend on one coaxed judge family; M1's deterministic leg is confounded (above).

## COMPOSITE SCORECARD (pilot + this run)

- grounding / real anchors:            ENGINE WINS (3.5x, deterministic, un-gameable)  [pilot P-A]
- downstream plan block-rate:          NEUTRAL at K=10                                  [pilot S]
- candidate-set diversity:             NO ENGINE ADVANTAGE (M1 confounded-lean-free, M2 null, M3 symmetric)
- pick novelty (blind judges):         FREESTYLE WINS 27-13                             [M4]
- pick value (blind judge BET):        FREESTYLE WINS 31-8 — but structurally blind to the
                                       one dimension the engine provably wins (groundedness)

## WHAT FOLLOWS (design consequences, not spin)

1. The engine's honest positioning: a GROUNDING machine, not a creativity machine. The claim it
   supports is "my ideas name real seams and carry executable falsifiers", not "my ideas are more
   novel". Update the skill's self-description if it overclaims.
2. If pick-stage novelty matters: score novelty as an explicit separate axis (bin-coverage /
   cluster count of the SET reported beside the pick; optionally a tail-pick mode under
   stagnation), never fuse it into the receipts-ranked winner.
3. The right value test is EXECUTION-grounded, not judge-bet: author both arms' picks into plans
   and run the real wire/gate checkers (the pilot's P-A already approximates this and the engine
   wins there). A judge bet cannot see groundedness; stop asking it to.
4. For the DeepReason thread: this is the "recover vs generate" question answered at one regime —
   the within-model diversity machinery added nothing measurable single-shot; the world-coupled
   leg (anchor verification) remains the only proven win. Publishing this null to Aaron is
   measure-not-settle in its most credible form.

Raw: results.json + per-call cache in the session scratchpad novelty_ab/ (gitignored scratch);
harness .claude/tools/conjecture_novelty_ab.py reproduces from the pilot dir + seed 42.
