Source: Blog post — "Code Execution as Reasoning"
URL: https://raw.works/code-execution-as-reasoning/
Author: Raymond A. Weitekamp
Date: May 11, 2026
Reliability: MEDIUM (primary author, measured benchmarks, but self-reported; not peer-reviewed)
Related: "RLMs are the new reasoning models" (raw.works, Apr 20, 2026)
Related paper: arXiv:2604.14140 (LongCoT benchmark)
Related: arXiv:2512.24601 (RLM paper, already mapped at workflow/papers/rlm-interface.md)

CENTRAL THESIS
Results matter more than mechanism. LLMs do not need to reason in latent space if code execution
produces better outcomes. The "harness" (tool infrastructure + REPL) determines performance on
extended reasoning tasks more than the model itself.
Direct quote: "it's not (just) the model, it's the harness."

CORE MECHANISM: THE DESIRE PATH
Key concept: provide LLMs the Python tools they "naturally tried to use" in prior experiments,
rather than forcing latent-space-only constraints.
"Desire path": the instrument-native execution path the model prefers when given the choice.
Concrete implementation: DSPy.RLM (dspy.RLM, v3.1.3) providing a persistent Pyodide Python REPL
inside the RLM iteration loop. The model generates and executes Python code to inspect, slice,
compute over the task domain. The REPL is stateful across iterations within one task.

RELATION TO RLMs
RLMs are the scaffolding layer enabling code-execution-as-reasoning. The mechanism is:
  context -> bound as Python variable in REPL -> model generates code to query/slice it
  -> code execution returns results -> model iterates until answer ready
The post validates and extends arXiv:2512.24601's architecture (see rlm-interface.md):
RLM's rlm_query recursion + Pyodide REPL is exactly the substrate; DSPy.RLM is a higher-level
wrapper over the same pattern.

RELATION TO CODING AGENTS (Claude Code-style)
Key claim: "coding agents are useful for long-horizon tasks that don't present themselves as
coding problems." Code execution generalizes beyond code-centric tasks to chemistry, math, logic.
The Codex CLI result (GPT-5.5 xhigh) is cited as the strongest evidence: 79.6% on LongCoT-Mini
using a persistent sandbox loop, outperforming pure-latent-space models by 30+ percentage points.
Weitekamp positions this as: "Codex's persistent in-sandbox Python loop is grinding harder on
chemistry and math" — the tool loop replaces the need for long unbroken latent reasoning chains.

BENCHMARK RESULTS (MEASURED by Weitekamp on LongCoT and LongCoT-Mini)
Primary benchmark: LongCoT-Mini (500 questions) and full LongCoT (~2000 questions, 1995 used)

LongCoT-Mini (500 questions):
  Codex CLI + GPT-5.5 xhigh:  398/500 = 79.6%   MEASURED
  Opus 4.7 + DSPy.RLM:         377/500 = 75.4%   MEASURED
  Sonnet 4.5 + DSPy.RLM:       227/500 = 45.4%   MEASURED
  Sonnet 4.5 vanilla (no tools): 13/500 = 2.6%   MEASURED
  GPT-5.2 (paper baseline):     ~194/500 = ~38.7% VERIFY (search-derived, not directly stated here)

Full LongCoT (1995 questions):
  Codex CLI + GPT-5.5 xhigh: 1446/1995 = 72.5%   MEASURED
  Top public leaderboard at time of post: ~25.12% (paper-era, no-tool models)
  Codex CLI result is approximately 3x top published leaderboard score.

Per-domain breakdown (Codex CLI on full LongCoT, contradicting paper's "zero" prediction):
  Mathematics: 69.5%
  Chemistry: 41.0%

REFUTATION OF PAPER CLAIM
LongCoT paper (arXiv:2604.14140) predicted "compositional domains (Math, Chemistry) remain at zero
for all models." Weitekamp's May 2026 results directly contradict this for tool-augmented agents.
Author framing: "the walls aren't walls; with a stronger model and a more aggressive tool loop,
they're just programs."
CLASSIFICATION: paper's prediction was about latent-space-only models. Weitekamp's results use
code execution. The claims are not strictly contradictory but the paper understated ceiling.

HARNESS DETAILS
  Model A: DSPy.RLM (dspy==3.1.3), Pyodide REPL, max_iterations=50
  Model B: Codex CLI (OpenAI), persistent sandbox, "xhigh" reasoning effort
  Baseline: raw Anthropic SDK, single user message, no tools, max_tokens=64000
  Cost (Sonnet 4.5): vanilla=$31 for 500q, RLM=$621 for 500q (~20x multiplier)

KEY NUMBERS TO INGEST (for ELAI planning)
  Harness multiplier on Mini: 45.4% / 2.6% = 17.5x accuracy gain for RLM over vanilla (Sonnet 4.5)
  Code-execution ceiling (Mini): ~79.6% (Codex CLI), ~75.4% (RLM)
  Cost multiplier: ~20x for RLM vs vanilla
  Tool-loop vs paper ceiling: 3x improvement over best paper-era (no-tool) scores on full benchmark

LIMITATIONS AND CAVEATS
  - Self-reported by benchmark runner, not peer-reviewed
  - "Preliminary" label on some Qwen3.5 numbers (from X posts, not this article directly)
  - Codex CLI "xhigh" is not a publicly documented parameter — may not be reproducible exactly
  - Cost data only for Sonnet 4.5; Codex CLI and Opus 4.7 cost not stated
  - No error breakdown by domain for Opus 4.7 or Codex CLI

IMPLICATIONS FOR RLM ARCHITECTURAL DECISIONS
  1. Code execution >> latent space for structured/compositional tasks at this token horizon
  2. The model choice matters less than the harness architecture once tools are present
  3. Graph-structured dependencies (type inference, flow algorithms) remain hard even with tools
     (Weitekamp Apr 2026 data: 0/75 on Hindley-Milner + MaxFlow-MinCut even with RLM)
  4. Chemistry and math are tractable with code execution (contradicts paper's architecture)
  5. "Desire path" principle: give the model the tools it reaches for naturally
