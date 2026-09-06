# B-3 multi-cycle frontier A/B

K=1 paired tasks; cycles=3; six generated candidates/cycle/arm;
model `kimi-k2-7`; seed 20260712.

## Binding gate

**REFUTED** — SUPPORTED only if paired archive-novel VDC and execution-value deltas are both positive with CI95 lower bounds >0

- archive-novel validated discovery delta: -3.0000, CI95 [-3.0000, -3.0000]
- execution-value delta: 0.000000, CI95 [0.000000, 0.000000]
- frontier-yield delta (secondary): -0.824953

| arm | tasks | novel validated/task | best value delta | frontier yield | tokens | failed cycles | valid candidates |
|---|---:|---:|---:|---:|---:|---:|---:|
| FDL archive-conditioned | 1 | 7.000 | 0.022105 | 1.397764 | 4108 | 0 | 18 |
| freestyle-with-memory | 1 | 10.000 | 0.022105 | 2.222716 | 3599 | 0 | 18 |

## Integrity

- Every task ran both arms on the same hidden instances with identical cycles, candidate caps, model,
  token cap, DSL, execution evaluator, and structured score memory.
- FDL cycle 1 was direct; later cycles used archive-conditioned operator rotation. Raw candidate prose
  never entered parent prompts; only strategy signature, score, validated bit, family, and failed edges.
- Exact DSL signature is the mechanism-equivalence instrument; execution receipts are BLAKE3 hashes.
- Validation means beating the strongest member of the frozen two-strategy no-local-search portfolio by >0.005.
- Both archive-relative novelty and execution value had to beat control with positive paired CI95 lower bounds.
