# Conjecture-engine benchmark — K=10 pilot run (2026-07-11)

Protocol: workflow/reasoning/conjecture-engine-benchmark-plan-2026-07-11.md.
Harness: .claude/tools/conjecture_bench_run.py + conjecture_bench_score.py.
Raw run dir (gitignored scratch): bench_k10/ (30 plans + manifest.json + scored.json).

## VERDICT (plain)

PARTIAL / MECHANISM-SPECIFIC. The engine measurably improves the ONE thing it is built to improve —
grounding ideas in REAL code anchors — by a large, un-gameable margin. It does NOT show a downstream
reduction in auditor block-rate at this scale, and the structural gate-pass difference is a formatting
artifact, not signal. So: the engine's designed mechanism works; the broader "engine-picked ideas make
better plans overall" claim is NOT supported at K=10. This is a pilot, not a settled number.

## SETUP

- K=10 real ELAI improvement triggers (context-lifecycle, wire-dark rate, gate-tax, case-reuse, routing
  economics, hallucination/CGG, triage unsoundness, token-ledger, perception gateway, compaction-resume).
- 3 arms: A1 freestyle-1, A2 freestyle-N (best-of-10, naive pick), A3 engine (scripted 4-stage).
- Generator + author = kimi-k2-7. Auditor panel = glm-5-2 + kimi-k2-7. (swe-1-7 and glm-5-2 are AGENTIC
  on the Devin bridge — they emit tool_use / refuse on generation prompts; kimi-k2-7 is the only reliably
  prose model, so it overrode the swe-1-7 preference. glm-5-2 is coaxed to prose via a tool_use handler
  for the audit leg only.)
- 30/30 units completed, 0 errors. All Devin (free), serial (the bridge is a serial-queue proxy).

## RESULTS

```
arm   n  anchorReal gatePass  blockR strictR meanBlks
A1   10       0.2      0.9      0.9     0.3      2.0
A2   10       0.4      1.0      1.0     0.1      2.3
A3   10       0.7      0.3      0.9     0.1      2.3

A3 vs A1 (block-count): engine_fewer=3 baseline_fewer=6 ties=1  delta=+0.3  CI95=[-0.8, 1.3]
A3 vs A2 (block-count): engine_fewer=5 baseline_fewer=3 ties=2  delta= 0.0  CI95=[-0.9, 1.0]
```

## READING (three metrics, three different stories)

1. anchorReal (P-A, the un-gameable deterministic core) — CLEAR WIN.
   The engine's PRODUCTION_HOOK named a REAL ELAI DEF-symbol on 7/10 triggers, vs 2/10 freestyle-1 and
   4/10 freestyle-N. 3.5x over one-shot freestyle. This is the engine's designed value: its stage-2
   anchor check verifies target_units against elai.map, that grounding flows into the plan, and an
   INDEPENDENT stricter check (the DEF-symbol set, not the engine's lenient substring check) confirms
   the named functions are real. Freestyle hallucinates plausible-but-fake seams (route_context,
   evaluate_merge_gates) — exactly the wire-dark failure mode this codebase fights. The engine cuts that.

2. gatePass (P-B, structural plan-gate) — ARTIFACT, DISCOUNTED.
   A3 = 0.3 looks like a loss but is a formatting quirk: the a3 plans fail plan_gate almost entirely on
   P6 ("a runnable command line must follow the VERIFY marker") because kimi wrote VERIFY inline
   (`VERIFY: cargo test ...`) rather than on the next line. Same author, same template as A1/A2; the
   quirk just correlated with the arm. Not an idea-quality signal.

3. block-count (S, auditor panel) — NEUTRAL / NOISY.
   Engine 2.3 vs freestyle 2.0/2.3; both A/B CIs straddle 0; inter-auditor agreement is low (strict rate
   0.1-0.3 — glm and kimi flag different categories). The panel does NOT separate the arms. The K=2
   verbosity confound (a3 2.5 vs 1.5) was largely controlled by the concise-author instruction (now
   2.3 vs 2.0). No reliable evidence the engine reduces auditor blocks.

## CAVEATS (do not oversell)

- K=10 is a PILOT. Effect sizes + bootstrap CIs only; no p<0.05 claim. The anchor win is large enough to
  survive the small N (7/10 vs 2/10), the block-count result is genuinely inconclusive (CI straddles 0).
- Generator is kimi-k2-7, NOT swe-1-7 (forced by the bridge's agentic behavior). A different generator
  could shift absolute numbers; the RELATIVE anchor gap (engine verifies, freestyle does not) is
  structural and should hold.
- A3 is a SCRIPTED faithful re-implementation of the 4-stage protocol, not the Claude-run skill
  (reproducibility over ecological validity). The UNIT-5 smoke separately confirmed the skill runs.
- The auditor panel is not a strong cross-family judge (glm coaxed from an agentic model; kimi shares the
  generator family). The DETERMINISTIC spine (P-A) is the load-bearing metric precisely because it does
  not depend on auditor quality.

## WHAT A REAL NUMBER WOULD NEED (a later plan, not this one)

- Larger K (>=30) for the block-count question to leave the noise floor.
- A stronger, genuinely cross-family auditor (a paid strong model, or a small human-graded set) — the
  free-Devin panel is too noisy to settle the "fewer blocks" claim.
- Fix the VERIFY-inline formatting in the author template so P-B becomes a fair structural signal.
- Optionally the heavier real downstream measure: author full wargame plans, run them through the actual
  auditor + wire-checker, compare true BLOCK rates (the gold-gold standard the plan named).

## BOTTOM LINE

The engine does what it claims at the mechanism level (real-anchor grounding, strong + un-gameable). The
larger "better plans overall" claim is unproven at this scale — honestly neutral, not a win and not a
loss. Worth keeping for the grounding value; not worth claiming a headline number until a larger,
stronger-judge run lands.
