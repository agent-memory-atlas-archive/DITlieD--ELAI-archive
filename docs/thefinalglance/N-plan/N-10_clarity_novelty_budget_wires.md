# N-10: clarity / novelty -> budget wires — type the passive detectors once, wire them to the resource deciders

- Track: TIER 3 (MEDIUM-HIGH) | lever: active-resource-allocation | finding: W-15+W-24 | grounding: STRONG | nature: ACTIVATION | effort: M | WAVE 6

- Premise: four passive clarity detectors + two novelty detectors exist, but NOTHING feeds the measured ambiguity/novelty of THIS task into how much research/reasoning it gets. They only route to a consent surface (grill offer) or a human queue. Turn four passive detectors into an active resource-allocation loop — pure composition, all inputs exist.

## Steal/build
Type the signals ONCE and declare consumers:
- AmbiguityScore (shared type) from B-38 continuous ambiguity (4 dims + floors + streak) + B-40 critic findings + C-29 intent pairs + J-32 front-door confidence;
- NoveltyScore (shared type, joined) from D-15 NoveltyClass (labels CLAIMS) + Phase-4F/B-21 novel-step detection (labels PLAN STEPS).
Consumers: research-tree depth (4D Simple/Moderate/Complex breadth/depth) + reasoning-effort tier (J-84) + decomposition node-sizing (J-39 bound-and-escalate) + grill/clarify offer (J-79). High-ambiguity OR off-map task -> deeper research + more clarify + smaller node bound + earlier escalation, not just a chip. Ambiguity = "we don't understand the ask"; novelty = "the ask is off our map" — both buy depth.
EXCLUDED: re-detecting (wire the existing detectors); a new detector; a non-deterministic depth choice (the mapping score->budget is deterministic, TD-3).

## Existing ground truth (PLAN-asserted; executor re-greps per W-7)
- B-38 ambiguity score, B-40 critic, C-29 intent pairs, J-32 front-door confidence exist; feed ONLY the J-79 grill offer today (W-15).
- D-15 NoveltyClass (Search/DistantSearch/DiscoveryCandidate->human) labels claims; 4F/B-21 novel-step labels plan steps; neither feeds budgets (W-24).
- 4D research-tree adaptive complexity + J-84 difficulty proxy + J-39 node-admission pick depth from OTHER inputs.
- N-5 intensity resolver is the natural home for the effort-tier consumer (the two compose).

## REVISED scope (W-10 tiered)
- SHIP NOW (Tier-1): the AmbiguityScore + NoveltyScore shared types + wiring them to 2-3 consumers (4D depth + J-84 effort via N-5 + J-79 offer). Deterministic score->budget mapping.
- DEFER (Tier-2): the J-39 node-sizing consumer (lands with the decomposition unit).

## Integration
- crates/orchestrator research-tree + effort-tier (via N-5) + node-admission + grill-offer seams.
- frontend_pair: N/A (internal resource allocation), unless the depth choice is surfaced on the canvas; justified.

## WIRING_CONTRACT
produces_symbols:
  - the AmbiguityScore + NoveltyScore shared types (joined) + the deterministic score->budget mapping
  - the wiring to the research-depth / effort-tier / node-size / grill-offer consumers
consumes_symbols:
  - B-38 / B-40 / C-29 / J-32 (ambiguity signals) + D-15 / 4F / B-21 (novelty signals)
  - 4D research-tree / J-84 effort (via N-5) / J-39 node-admission / J-79 grill offer
reachable_from:
  - a live task where the research-tree / effort tier is chosen  [PRODUCTION planning seam]
  - a #[cfg(test)] caller NEVER satisfies this (R7)
stub_ban:
  - NO re-detection (wire the shipped detectors)
  - NO non-deterministic score->budget mapping (TD-3)
  - NO high-ambiguity task getting baseline depth
frontend_pair: N/A (internal allocation); justified.
observable_effect:
  trigger: "a deliberately vague OR off-map task vs a clear baseline task"
  expect_ssb: "reuse the shipped ambiguity (B-38) / novelty (D-15) rows; no new type"
  expect_artifact: "research depth + effort tier are HIGHER for the vague/novel task, reading the same scores; NEGATIVE: a high-ambiguity task that gets baseline depth must FAIL"

## Grounding
- interlace doc W-15 (ambiguity never feeds budgets), W-24 (two novelty detectors, novelty never feeds budgets), TIER-3 item 10; NOTE W-15 extension (J-39 fourth customer).
- N-5 (the effort-tier consumer is the intensity resolver).

## COMPOSES-WITH
B-38/B-40/C-29/J-32, D-15/4F/B-21, 4D/J-84/J-39/J-79, N-5. Re-implements NONE (GD-1). Turns passive detectors into an active resource-allocation loop.

End of N-10.
