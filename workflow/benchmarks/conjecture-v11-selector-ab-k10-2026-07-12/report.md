# Conjecture v1.1 NOVEL-CREDIBLE selector A/B

K=10 shared candidate sets; generator `kimi-k2-7`; NLI `glm-5-2`; pair panel `glm-5-2, kimi-k2-7`.

## Profile gate

**SUPPORTED** — REFUTED at prior 13/40 engine-share margin or worse; SUPPORTED only on strict vote win

- novelty votes: engine 22 / baseline 14 / ties 4
- value votes: engine 6 / baseline 30 / ties 4
- prior refutation threshold: engine share <= 13/40 and baseline share >= 27/40
- trigger failures: 0

## Integrity

- Both selectors picked from the identical ten-card set per trigger.
- Generation was one direct self-coordinating call; raw archive prose never entered it.
- Two seeded NLI orderings produced versioned equivalence verdicts; disagreement degraded to UNCERTAIN.
- Pairwise ideas used the same compact field shape and randomized A/B labels.
- Model calls are hash-keyed and cached under the run directory.
