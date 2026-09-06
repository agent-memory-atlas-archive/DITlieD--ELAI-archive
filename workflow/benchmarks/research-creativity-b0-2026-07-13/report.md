# Q-10 B-CREATIVITY-0 research-conditioned creativity

K=30 paired tasks; five arms; model `kimi-k2-7`; bootstrap 5000 samples seed 20260712.

## Joint verdict: INVALID

- failed conditions: [1, 2, 3, 4, 5, 6, 7, 8, 10, 11]
- manifest/hash/hidden/support/token violations: 30

SUPPORTED requires all eleven conditions (prereg 5.6): RELATIONAL_PACKET beats VERIFIED_EVIDENCE/DIRECT/MATCHED_SCAFFOLD/SHUFFLED_PACKET on BOTH P1 and P2 with CI95 lower bounds above zero, guards G1/G2a/G2b/G3 non-inferior within -0.05, zero violations.

## G3 total-cost columns (prereg 5.4)

Total cost per arm = research + packetization + generation(input+output tokens) + execution; tokens/calls/latency/executions are reported separately. Research and packetization are identical across the matched arms by construction and are recorded regardless.

| arm | research_tok | packetization_tok | gen_in_tok | gen_out_tok | gen_calls | gen_latency_s | executions | total_cost |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| DIRECT | 0 | 0 | 5843 | 22415 | 30 | 268.642 | 179 | 28437.000 |
| MATCHED_SCAFFOLD | 6900 | 216491 | 15696 | 25005 | 30 | 307.373 | 180 | 264272.000 |
| VERIFIED_EVIDENCE | 6900 | 216491 | 14372 | 35368 | 30 | 428.395 | 171 | 273302.000 |
| RELATIONAL_PACKET | 6900 | 216491 | 14357 | 36595 | 30 | 430.306 | 179 | 274522.000 |
| SHUFFLED_PACKET | 6900 | 216491 | 14357 | 32263 | 30 | 364.181 | 179 | 270190.000 |

## Reporting note (condition 9 scope, prereg 5.4)

G2a/G2b are shared-packet input properties; their paired deltas are structurally 0 and only G1 gives condition 9 bite.

## SOURCE POOL PROVENANCE (orchestrator condition C1, DECISION 2026-07-13)

The frozen source pool for this run is AUTHORED SYNTHETIC FIXTURE DATA (6 records, quality="synthetic-fixture"), not literature-derived evidence. It was approved as OPTION (a) in workflow/reasoning/q10-sp1-blocker-2026-07-12.md; B-CREATIVITY-0 is the mechanism falsifier only.

## EXTERNAL VALIDITY (orchestrator condition C2)

External validity to REAL research packets is deferred to B-CREATIVITY-1 (real ELAI fixtures) by design; nothing in this run's outcome generalizes beyond the locked synthetic benchmark tasks and model.

## REQUIRED NULL/INVALID LANGUAGE (prereg sec 9)

The joint verdict is INVALID. Exact failed conditions: 1-8 (both primaries, all four causal comparisons: every RELATIONAL_PACKET-minus-control CI95 lower bound and point estimate <= 0), 10 (G3 cost-normalized frontier yield non-inferiority lower bound -0.9456 << -0.05), 11 (30 counted violations). Condition 9 passed (G1 lower 0.0056, G2a/G2b 0.0, all >= -0.05). All 30 condition-11 violations are one leg: the W3 REAL-token envelope re-check — the frozen whitespace-proxy padding matched all 30 tasks exactly (0 proxy violations) but the real kimi-k2-7 tokenizer input_tokens spread across the four matched arms was 5-13% per task, above the 2% tolerance, counted fail-closed. Per prereg 5.6 disposition: fix the instrument (padding renderer must match the REAL tokenizer envelope) under a new preregistration version; do not interpret arm outcomes. All negative artifacts preserved in this directory.
