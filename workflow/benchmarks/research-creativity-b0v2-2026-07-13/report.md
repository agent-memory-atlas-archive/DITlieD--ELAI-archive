# Q-10 B-CREATIVITY-0 research-conditioned creativity

K=30 paired tasks; five arms; model `kimi-k2-7`; bootstrap 5000 samples seed 20260712.

## Joint verdict: INVALID

- failed conditions: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]
- manifest/hash/hidden/support/token violations: 4

SUPPORTED requires all eleven conditions (prereg 5.6): RELATIONAL_PACKET beats VERIFIED_EVIDENCE/DIRECT/MATCHED_SCAFFOLD/SHUFFLED_PACKET on BOTH P1 and P2 with CI95 lower bounds above zero, guards G1/G2a/G2b/G3 non-inferior within -0.05, zero violations.

## G3 total-cost columns (prereg 5.4)

Total cost per arm = research + packetization + generation(input+output tokens) + execution; tokens/calls/latency/executions are reported separately. Research and packetization are identical across the matched arms by construction and are recorded regardless.

| arm | research_tok | packetization_tok | gen_in_tok | gen_out_tok | gen_calls | gen_latency_s | executions | total_cost |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| DIRECT | 0 | 0 | 5920 | 21981 | 30 | 229.369 | 156 | 28057.000 |
| MATCHED_SCAFFOLD | 6900 | 208548 | 16858 | 25108 | 30 | 236.594 | 175 | 257589.000 |
| VERIFIED_EVIDENCE | 6900 | 208548 | 14280 | 40564 | 30 | 370.396 | 169 | 270461.000 |
| RELATIONAL_PACKET | 6900 | 208548 | 15665 | 30604 | 30 | 293.647 | 172 | 261889.000 |
| SHUFFLED_PACKET | 6900 | 208548 | 15637 | 33691 | 30 | 310.066 | 163 | 264939.000 |

## Reporting note (condition 9 scope, prereg 5.4)

G2a/G2b are shared-packet input properties; their paired deltas are structurally 0 and only G1 gives condition 9 bite.

## SOURCE POOL PROVENANCE (orchestrator condition C1, DECISION 2026-07-13)

The frozen source pool for this run is AUTHORED SYNTHETIC FIXTURE DATA (6 records, quality="synthetic-fixture"), not literature-derived evidence. Approved as OPTION (a) in workflow/reasoning/q10-sp1-blocker-2026-07-12.md; B-CREATIVITY-0 is the mechanism falsifier only.

## EXTERNAL VALIDITY (orchestrator condition C2)

External validity to REAL research packets is deferred to B-CREATIVITY-1 (real ELAI fixtures) by design; nothing in this run's outcome generalizes beyond the locked synthetic benchmark tasks and model.

## REQUIRED NULL/INVALID LANGUAGE (prereg sec 9) — RUN 2, preregistration VERSION 2

The joint verdict is INVALID. Exact failed conditions: 1-8 (both primaries: every RELATIONAL_PACKET-minus-control CI95 lower bound <= 0; P1 points negative vs DIRECT/MATCHED_SCAFFOLD/VERIFIED_EVIDENCE, +0.0063 vs SHUFFLED_PACKET only; P2 points negative vs all but DIRECT +0.0004), 9 (G1 validity non-inferiority lower bound -0.0833 < -0.05), 10 (G3 lower bound -0.9177 << -0.05), 11 (4 counted violations).

The v2 pad-match itself WORKED: sealed padded REAL-token spread max 1.23% (T25), mean 0.58%, 0/30 tasks over the 2% tolerance at preparation (244 measurement calls, sealed). The 4 condition-11 violations are the in-run W3 re-check firing on tasks T03/T27/T29/T30 because the kimi-k2-7 lane's usage.input_tokens is NOT a deterministic function of the prompt text: the byte-identical sealed prompt returned e.g. 1764 input tokens at the prep measurement call and 419 at the scored call (T03 VERIFIED_EVIDENCE; T03 other arms 1761-1763 vs prep-sealed ~520). This is lane-telemetry nondeterminism (prompt-cache-style accounting), not padding drift; the frozen rule counts it fail-closed regardless. Per prereg 5.6 disposition: instrument-side fix (a telemetry-stable envelope measurement, e.g. a frozen local tokenizer as the authoritative envelope meter with lane usage demoted to a report column) requires a new preregistration version; do not interpret arm outcomes. Independently of condition 11, conditions 1-10 failed on the measured data (REFUTED-shaped primaries and guards). All negative artifacts preserved in this directory; run 1 (INVALID, whitespace-proxy defect) is archived at workflow/benchmarks/research-creativity-b0-2026-07-13/.
