Source: Blog post — "RLMs are the new reasoning models"
URL: https://raw.works/rlms-are-the-new-reasoning-models/
Author: Raymond A. Weitekamp
Date: April 20, 2026
Reliability: MEDIUM (author of raw.works RLM benchmarking series; self-reported numbers; not peer-reviewed)
Related: arXiv:2512.24601 (RLM paper, already mapped at workflow/papers/rlm-interface.md)
Related: arXiv:2604.14140 (LongCoT benchmark, mapped at workflow/papers/longcot-bench-2604.14140-interface.md)
Related: "Code Execution as Reasoning" (raw.works May 11, 2026, mapped at code-execution-as-reasoning-rawworks-interface.md)

CENTRAL CLAIM
RLMs are the successor to reasoning models (o1-class), not merely an extension.
The unification claim: "reasoning and tool use were previously separate capabilities that RLMs unified."
Core mechanism: "the model treats its own prompt as an environment it can inspect, slice, and
recursively query. Context itself becomes the object of computation."
Operational definition: "a REPL in which the prompt is bound to a variable it can inspect,
slice, and partition programmatically."

HISTORICAL TRAJECTORY CITED
  2022: chain-of-thought reasoning (Wei et al.)
  Late 2022: ReAct (reasoning + tool use as separate steps)
  2023: tool-use standardization (function calling APIs)
  2024: reasoning models (o1) — latent-space extended compute, no external tools
  2025-2026: RLMs — collapse reasoning + tool use into one recursive abstraction
The author's argument: o1-class models extended the reasoning budget but kept context passive.
RLMs make context active: the model can programmatically query, slice, and recurse over it.

WHAT RLMs DO THAT REASONING MODELS DO NOT
  Reasoning models (o1/o3): more thinking tokens; context is still a static string
  RLMs: context is a Python variable; model issues code to query/slice it; can recurse
  Key capability: process inputs "up to two orders of magnitude beyond model context window"
  Recursive sub-calling: rlm_query(prompt) spawns a full child RLM; depth-bounded recursion
  (See rlm-interface.md for complete architectural spec from arXiv:2512.24601)

BENCHMARK NUMBERS CITED IN POST (MEASURED)
LongCoT (long-horizon reasoning):
  Qwen3.5-9B + RLM: ~15.69% on full LongCoT (approx 1.6x GPT-5.2's 9.83%)
  Qwen3.5-27B + RLM: ~22.18% on full LongCoT (approx 2x GPT-5.2)
  Claude Sonnet 4.5 + RLM: 45.4% on LongCoT-Mini vs 2.6% without recursion
  WARN: Qwen3.5 numbers labeled "preliminary" in X post; full benchmark slice unspecified

Oolong (long-context aggregation):
  Claimed: RLMs process inputs "up to two orders of magnitude beyond model context windows"
  Claim status: UNVERIFIED (no specific Oolong score quoted in this post; only narrative claim)
  NOTE: author also notes "Coding Agents paper outperformed published RLM baselines on Oolong"
  suggesting RLMs are not definitively optimal on long-context aggregation specifically

LongMemEval (memory/interaction):
  Claimed: 87-95% accuracy with structured scaffold
  WARN: no model specified, no harness spec, range not explained - treat as illustrative not precise

ACKNOWLEDGED LIMITATIONS (author)
  Cost: RLM costs ~20x vanilla (confirmed in Apr 2026 Sonnet 4.5 experiment: $621 vs $31/500q)
  Unpredictable execution time (unbounded iterations up to max_iterations)
  Models do not naturally decompose problems recursively without scaffolding

HARNESS IMPLEMENTATIONS RECOMMENDED
  alexzhang13/rlm (GitHub)
  DSPy.RLM (dspy >= 3.1.3)
  ax-llm/ax
  rawwerks/rlm-cli
  rawwerks/ypi
  WARN: availability and maintenance status not verified for all five; verify before use

POSITIONING vs LONG-CONTEXT MODELS
The post argues RLMs are not a scaling variant of long-context models:
  Long-context models: larger context window, still passive; scores on LongCoT ~0% for small,
  <=10% for frontier (e.g. Gemini 3 Pro 6.1%, GPT-5.2 9.83%)
  RLMs: active context manipulation; 15-45% depending on model/task/difficulty tier
  Author's framing: "recursive scaffolds can manufacture reasoning performance not visible in
  the base model alone" — confirmed by Sonnet 4.5: 2.6% -> 45.4% (17.5x) on Mini
  Caution: these numbers are Mini (easier), not full benchmark

HONEST EPISTEMIC NOTE IN THE POST
Author acknowledges RLM was "the first clearly articulated expression of a larger family"
rather than claiming RLMs are definitively optimal. The Coding Agents result on Oolong is
cited as evidence that "coding agents" (Claude Code-style persistent sandbox) may match or
exceed published RLM baselines on some long-context tasks. This is intellectually honest and
relevant for ELAI: the exact architecture (RLM vs coding agent vs hybrid) is still open.

RELATION TO arXiv:2512.24601 (the RLM paper, rlm-interface.md)
This post is the same author's popularization and follow-on benchmarking of the 2512.24601 paper.
The architectural spec is in the paper. This post supplies:
  - The motivating framing (RLM as o1 successor, not supplement)
  - Independent LongCoT benchmark results not in the original paper
  - Harness comparison (DSPy.RLM vs coding-agent style)
  - Cost data
Do NOT treat this post as the primary source for RLM architecture. Use rlm-interface.md for that.

IMPLICATIONS FOR ELAI
  1. "Reasoning model" framing is superseded: the 2024 o1-era architecture is one step behind
  2. Active context manipulation (code over variables) outperforms passive retrieval at scale
  3. Small models (9B, 27B) + RLM beat GPT-5.2 on a hard long-horizon benchmark (preliminary)
  4. The harness matters more than the model for long-horizon tasks
  5. The RLM vs coding-agent distinction may be collapsing (both use persistent code execution)
