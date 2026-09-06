# B-3 multi-cycle frontier A/B

K=30 paired tasks; cycles=3; six generated candidates/cycle/arm;
model `kimi-k2-7`; seed 20260712.

## Binding gate

**REFUTED** — SUPPORTED only if paired archive-novel VDC and execution-value deltas are both positive with CI95 lower bounds >0

- archive-novel validated discovery delta: -0.4667, CI95 [-1.0333, 0.0667]
- execution-value delta: -0.000105, CI95 [-0.000223, -0.000016]
- frontier-yield delta (secondary): -0.343340

| arm | tasks | novel validated/task | best value delta | frontier yield | tokens | failed cycles | valid candidates |
|---|---:|---:|---:|---:|---:|---:|---:|
| FDL archive-conditioned | 30 | 7.100 | 0.010945 | 0.985310 | 207201 | 0 | 539 |
| freestyle-with-memory | 30 | 7.567 | 0.011050 | 1.328650 | 148119 | 0 | 539 |

## Integrity

- Every task ran both arms on the same hidden instances with identical cycles, candidate caps, model,
  token cap, DSL, execution evaluator, and structured score memory.
- FDL cycle 1 was direct; later cycles used archive-conditioned operator rotation. Raw candidate prose
  never entered parent prompts; only strategy signature, score, validated bit, family, and failed edges.
- Exact DSL signature is the mechanism-equivalence instrument; execution receipts are BLAKE3 hashes.
- Validation means beating the strongest member of the frozen two-strategy no-local-search portfolio by >0.005.
- Both archive-relative novelty and execution value had to beat control with positive paired CI95 lower bounds.
