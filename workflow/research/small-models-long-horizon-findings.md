# Why small (<24GB VRAM) models lose to frontier / 8xH100 models on agentic coding

Grounded research synthesis. Date assembled: 2026-06-14. Context: prompted by the
Agents' Last Exam (ALE) leaderboard (agents-last-exam.org) analysis, where the best
self-hostable open-weights model (DeepSeek V4 Pro, 12.4% pass) scores roughly HALF of
Codex+GPT-5.5 (24.0% pass). Question: WHY do small models lose -- context space, raw
reasoning limit, or something else?

All claims below are grounded in the cited 2026 papers (web-verified 2026-06-14), not
training memory. Where a point is inference rather than a direct paper finding, it is
labelled.

## TL;DR (the counterintuitive headline)

The dominant cause is NOT raw reasoning ("can't figure out the fix") and NOT primarily
context window size. It is EXECUTION RELIABILITY OVER A LONG HORIZON. Small models make
slightly more errors per step; on long multi-step tasks those errors compound
exponentially; and small models additionally SPIRAL after their own first mistake
(self-conditioning). Even when a small model has 100% single-step accuracy, larger
models still execute more steps correctly in sequence.

## Mechanism 1 -- error compounding (the main driver; execution, not reasoning)

Paper: "The Illusion of Diminishing Returns: Measuring Long Horizon Execution in LLMs"
(arXiv:2509.09677). Direct findings:
- Failures "arise from mistakes in EXECUTION, rather than an inability to reason."
- "Larger models can correctly execute significantly more turns EVEN WHEN small models
  have 100% single-turn accuracy."
- Marginal single-step accuracy gains compound into EXPONENTIAL gains in completable
  task length.

The math: if per-step accuracy is p and a task needs n dependent steps, success ~= p^n.
ALE tasks are hours-long with dozens-to-hundreds of tool calls (large n), so a tiny
per-step gap explodes:

  per-step acc   20-step   50-step   100-step
  99.5%          90%       78%       61%
  97%            54%       22%       5%
  93%            23%       2.6%      0.07%

A 99.5%-vs-97% per-step gap becomes 61%-vs-5% on a 100-step task. The small model is not
failing to reason out the fix; it gets most steps right, then one slipped tool call /
wrong path / mis-parsed output midway poisons everything downstream.

## Mechanism 2 -- self-conditioning (small models spiral, big models recover)

Same paper (arXiv:2509.09677), reinforced by "The Illusion of Procedural Reasoning:
Measuring Long-Horizon FSM Execution in LLMs" (arXiv:2511.14777):
- "Models become more likely to make mistakes when the context contains their errors
  from prior turns."
- Smaller models self-condition MORE severely: once an error is in their own context,
  P(next error) rises -- they spiral. Frontier models are more robust to their own past
  errors and recover.
- So small models do not just suffer passive p^n decay; they actively get WORSE after
  the first mistake.

## Mechanism 3 -- knowledge breadth (secondary, parameter-count linked)

Inference grounded in parameter-count literature + ALE task design: ALE spans 55
industries with real professional software. Fewer parameters = a thinner long-tail of
baked-in knowledge about specific tools/APIs/domains = more factual/API errors per step,
which feeds Mechanism 1. This is part of the 1.6T-param (DeepSeek V4 Pro) vs <24GB-model
gap.

## The user's three hypotheses, ranked by the evidence

1. "Pure reasoning limit -- can't figure out a fix" -> PARTIALLY true, but only for the
   hardest tasks. On ALE's `last-exam` split even GPT-5.5 scores ~3%; those need genuine
   reasoning small models lack. For the BULK of failures the research explicitly says it
   is NOT reasoning-inability.
2. "Lack of context space" -> a CONTRIBUTOR, not the main driver. Long trajectories
   stress small models (smaller effective windows, lost-in-the-middle degradation), but
   the gap persists with adequate context. Note: the ALE team's own "harness matters"
   blog found context-compaction RARELY EVEN TRIGGERED for frontier models -- the window
   was not the binding constraint there.
3. DOMINANT factor -> per-step execution reliability x horizon length + self-conditioning
   + narrower baked-in knowledge.

## Reconciliation with ELAI's own measured data

ELAI measured Qwen2.5-Coder 7B at 100% pass / 25K tokens on its own task set (see
memory: project_qwen3b_baseline). NOT a contradiction. Those are short, well-scoped tasks
(small n), where p^n stays high even at modest p. The small model is genuinely fine
there. It collapses specifically when the horizon is long AND broad -- exactly what ALE
stresses and the ELAI task set does not. Correct framing: "small models are bad at
long-horizon, many-step, broad-domain execution," NOT "small models are bad."

## What closes the gap (and what does not)

Helps:
- Extended thinking / CoT: paper found "thinking mitigates self-conditioning and enables
  execution of much longer tasks in a single turn." A reasoning-tuned small model claws
  back some horizon.
- Harness-level error recovery, checkpointing, and DECOMPOSITION into shorter sub-horizons
  (smaller n per unsupervised chain). This is the one lever a harness, not just the model,
  controls -- and is essentially what ELAI's orchestration + Creator-Verifier design is
  for.

Does NOT help much:
- Just bolting on a bigger context window. The bottleneck is reliability-per-step and
  error-recovery, not window size.

## Design implication for ELAI

The fix for running smaller/local models well is NOT a bigger model per se; it is
shortening the EFFECTIVE HORIZON so n-per-unsupervised-chain stays small: aggressive
decomposition, per-step verification, and cheap rollback/recovery after the first error
(before self-conditioning spirals). This is the architectural argument behind ELAI's
efficiency + trust direction (see workflow/reasoning/ELAI-EFFICIENCY-TRUST-DIRECTION-
2026-06-03.md) made concrete with external grounding.

## Sources (web-verified 2026-06-14)

- arXiv:2509.09677 -- "The Illusion of Diminishing Returns: Measuring Long Horizon
  Execution in LLMs" (PRIMARY: execution-not-reasoning, p^n compounding, self-conditioning,
  thinking mitigates).
- arXiv:2511.14777 -- "The Illusion of Procedural Reasoning: Measuring Long-Horizon FSM
  Execution in LLMs" (accuracy declines predictably with #actions + input length; self-
  conditioning dynamics).
- arXiv:2604.11978 -- "The Long-Horizon Task Mirage? Diagnosing Where and Why Agentic
  Systems Break."
- arXiv:2605.02572 -- "On Training Large Language Models for Long-Horizon Tasks: An
  Empirical Study of Horizon Length."
- ALE context: agents-last-exam.org/blogs/harness-matters (model accounts for ~3x the
  pass-rate spread of the harness; compaction rarely triggered).

## Caveats / not-fully-grounded

- The per-step accuracy figures in the compounding table are ILLUSTRATIVE (chosen to show
  the p^n shape), not measured values for specific models.
- Mechanism 3 (knowledge breadth) is inference from parameter-count literature + ALE task
  design, not a single direct citation.
- Open-weights-vs-proprietary classification of the 2026 model revisions (DeepSeek V4 Pro
  open; Qwen "Max"/"Plus" = proprietary API tiers) verified via web 2026-06-14 but moves
  fast; re-verify before relying on it.
