# Q-10 B-CREATIVITY-0 research-conditioned creativity

K=30 paired tasks; five arms; model `kimi-k2-7`; bootstrap 5000 samples seed 20260712.

## Joint verdict: REFUTED

- failed conditions: [1, 2, 3, 4, 5, 6, 7, 8, 10]
- manifest/hash/hidden/support/token violations: 0

SUPPORTED requires all eleven conditions (prereg 5.6): RELATIONAL_PACKET beats VERIFIED_EVIDENCE/DIRECT/MATCHED_SCAFFOLD/SHUFFLED_PACKET on BOTH P1 and P2 with CI95 lower bounds above zero, guards G1/G2a/G2b/G3 non-inferior within -0.05, zero violations.

## G3 total-cost columns (prereg 5.4)

Total cost per arm = research + packetization + generation(input+output tokens) + execution; tokens/calls/latency/executions are reported separately. Research and packetization are identical across the matched arms by construction and are recorded regardless.

| arm | research_tok | packetization_tok | gen_in_tok | gen_out_tok | gen_calls | gen_latency_s | executions | total_cost |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| DIRECT | 0 | 0 | 5837 | 25965 | 30 | 369.662 | 162 | 31964.000 |
| MATCHED_SCAFFOLD | 6900 | 233848 | 15954 | 36869 | 30 | 432.319 | 180 | 293751.000 |
| VERIFIED_EVIDENCE | 6900 | 233848 | 14712 | 37029 | 30 | 500.386 | 157 | 292646.000 |
| RELATIONAL_PACKET | 6900 | 233848 | 14735 | 26909 | 30 | 368.937 | 178 | 282570.000 |
| SHUFFLED_PACKET | 6900 | 233848 | 14735 | 27835 | 30 | 393.629 | 175 | 283493.000 |

## Reporting note (condition 9 scope, prereg 5.4)

G2a/G2b are shared-packet input properties; their paired deltas are structurally 0 and only G1 gives condition 9 bite.

## Meter drift (INFO, prereg v3)

Scored-time usage.input_tokens is recorded per arm; a deviation from the sealed padded_real_tokens is reported here as METER-DRIFT and is NEVER a condition-11 violation (byte-identical prompts are equal budgets -- the bytes are the budget).
- METER-DRIFT T21/MATCHED_SCAFFOLD: sealed 554 vs scored 1806 (drift 2.2599, plausible=False)

## Primary deltas (RELATIONAL_PACKET minus each control; 5000-sample paired CI95)

P1 = validated improving-strategy Recovery delta. P2 = best execution delta. SUPPORTED needs every control beaten on BOTH primaries with the CI95 lower bound strictly above 0. Every P1 delta is NEGATIVE (relational recovers fewer good strategies than every control, including the shuffled-mapping and evidence-only controls) and every P2 delta is ~0 with a CI lower bound below 0.

| contrast | P1 point | P1 CI95-low | P2 point | P2 CI95-low |
| --- | --- | --- | --- | --- |
| RELATIONAL vs DIRECT | -0.0458 | -0.1101 | +0.00114 | -0.00076 |
| RELATIONAL vs MATCHED_SCAFFOLD | -0.0459 | -0.0907 | -0.00012 | -0.00127 |
| RELATIONAL vs VERIFIED_EVIDENCE | -0.0292 | -0.0649 | +0.0000034 | -0.00133 |
| RELATIONAL vs SHUFFLED_PACKET | -0.0338 | -0.0881 | -0.00023 | -0.00152 |

Condition 10 (G3 cost non-inferiority) also failed: the relational arm costs ~8.8x DIRECT (282,570 vs 31,964 total-cost units, dominated by the 233,848-token packetization leg) with no primary win to justify it. This is not a near-miss; the mechanism slightly HURTS recovery and costs far more.

## C1 -- source-pool honesty (prereg DECISION (a))

The source_pool is a frozen 6-record SYNTHETIC-FIXTURE pool (labelled quality=synthetic-fixture), not real research sources. This is by design: B-CREATIVITY-0 is a MECHANISM falsifier on the exactly-enumerable 45-strategy synthetic space; real verified research sources enter the ladder at B-CREATIVITY-1. The synthetic pool controls the mechanism (does a research-shaped relational packet beat its controls?) without conflating source quality.

## C2 -- external validity (deferred)

This run measures the mechanism on a synthetic closed-strategy class ONLY. It does NOT measure whether ELAI is more creative on real problems, nor validate the shipped creativity layer. A REFUTED here means the research-conditioned relational-packet coupling does not clear the falsifier on this class; a different creativity mechanism is the honest next lever. Real-task value would have been B1's question and is not reached.

## Independent-verifier correction (2026-07-14, user-authorized)

The independent checker's --verify-hidden-isolation leg initially fired 2 hits on T16 (MATCHED_SCAFFOLD, VERIFIED_EVIDENCE) for the token `33.0`. Diagnosed as a CONFIRMED FALSE POSITIVE, not an answer leak:
- `33.0` is a common hidden float payoff carried by 16 OTHER tasks (T02, T05, ... T30); T16's OWN hidden data has zero `33.0` values, so T16's answer is not leaked.
- The `33.0` in T16's two flagged prompts is the scaffold's PUBLIC budget-fraction constraint ("budget approx 33.0% of total cost") -- exactly the two arms that embed the scaffold's target_constraints; DIRECT/RELATIONAL/SHUFFLED (which do not carry that text) were correctly clean.
- Root cause: verify_hidden_isolation unioned every task's hidden tokens into a GLOBAL set, whereas leakage is a PER-TASK property (a prompt must not leak ITS OWN task's answer) -- the same threat model the runner's per-task scan_for_hidden_leakage already used (which correctly reported 0).
Fix: verify_hidden_isolation now scans each raw prompt against only its own task's hidden tokens (token-independent DSL-signature and receipt-digest scans still run on every prompt). Regression test `test_cross_task_value_collision_is_not_a_violation` pins it (cross-task collision -> clean; same-task payoff -> still a violation). Suite 74 -> 75 green; wire-check 0 DARK; maintainability 0/0.
Integrity note: b0v3's manifest.json seals the PRE-FIX verifier blake3 in tool_module_hashes (the run was frozen under it). The fix is verifier-LOGIC only, touches no generation evidence, is strictly more correct (removes a false positive, adds no bypass), and was applied under explicit user authorization -- so the re-check over the untouched frozen evidence is legitimate. A later runner --preflight-frozen would report a benign tool-module-drift on the verifier line, reflecting exactly this documented correction.

## Verdict + permission-ladder outcome

VALID REFUTED (0 real violations across freeze/hidden-isolation/support-edges/token-envelope after the verifier correction). Per the locked permission ladder: REFUTED means the mechanism is dead for this synthetic task class -- NO B-CREATIVITY-1, NO skill/Rust/runtime change, NO paper-to-plan/idea-intake adoption. This is the formal verdict the two prior INVALID runs could not produce; the twice-observed negative advisory direction is now confirmed by a valid run.
