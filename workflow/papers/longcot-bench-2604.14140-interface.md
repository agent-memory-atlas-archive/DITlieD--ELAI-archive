Algorithm: LongCoT Benchmark (Long-Horizon Chain-of-Thought Reasoning Benchmark)
ArXiv: 2604.14140
Paper: LongCoT: Benchmarking Long-Horizon Chain-of-Thought Reasoning
Authors: Sumeet Ramesh Motwani, Daniel Nichols, Charles London + 20 co-authors (23 total)
Submitted: April 15, 2026
Reliability: HIGH (arXiv-verified, GitHub repo live, HF dataset live, MIT License)

Repo: https://github.com/LongHorizonReasoning/longcot
Dataset: https://huggingface.co/datasets/LongHorizonReasoning/LongCoT
Website: https://longcot.ai/
License: MIT (confirmed from GitHub README)

WHAT IT MEASURES
Long-horizon chain-of-thought reasoning: the ability to plan and execute a long, correct,
multi-step reasoning trace without error propagation. Isolated from step-level difficulty
(each individual step is tractable for frontier models in isolation). Failures reflect
long-chain management, not inability to solve individual sub-steps.
Contrasts with: GSM8K, MATH (short chains), needle-in-haystack (retrieval not reasoning).

TASK CONSTRUCTION
2,500 expert-designed problems. Five domains, 500 problems each:
  - Logic: planning, constraint satisfaction, backtracking (e.g. BlocksWorld, Sokoban, Sudoku, Hanoi)
  - Computer Science: algorithm simulation, program tracing (e.g. Hindley-Milner type inference, MaxFlow-MinCut)
  - Chemistry: molecular reasoning via SMILES strings
  - Chess: move evaluation, game reconstruction (e.g. 100 chess problems)
  - Mathematics: chained competition-style problems
Design invariant: short self-contained input, verifiable answer, solution requires navigating
a dependency graph of steps spanning tens to hundreds of thousands of reasoning tokens.
No tools or external scaffolding in the primary evaluation setting.

DIFFICULTY TIERS
LongCoT-Mini: 500 problems (easy subset, 100 per domain) - for rapid iteration
Full LongCoT: ~2,000 problems (medium + hard tiers)
The paper's primary reported numbers are on the full benchmark; mini numbers differ significantly.

METRIC
Primary: Accuracy = correct_answers / (correct + incorrect) (excludes formatting failures)
Secondary: Overall accuracy = correct_answers / total_questions
Error categories tracked: failures, formatting issues, correct
Average reasoning tokens per question: ~62K

HARNESS (how to run it - PORTABLE)
Runtime: Python via `uv` package manager
Inference: uv run python run_inference.py --difficulty longcot-mini --config oai_gpt52
Evaluation: uv run python run_eval.py responses/<file>.jsonl
Built-in configs: GPT-5.2, Claude Opus 4.6, Gemini 3 Pro, OpenRouter (DeepSeek, GLM, Grok, Kimi)
Custom models: add config file pointing to any OpenAI-compatible API
Portability verdict: OPEN and RUNNABLE. MIT license, standard Python deps (OpenAI/Anthropic/Google SDKs).

BASELINE MODEL PERFORMANCE (paper-reported, no tools/scaffolding)
These are MEASURED numbers from the paper on full LongCoT:
  GPT-5.2: 9.83% (top published baseline at paper release)
  Gemini 3 Pro: 6.08%
  Grok 4.1 Fast Reasoning: 2.04%
  Claude: not in paper's primary leaderboard at release (UNVERIFIED - not listed in available data)

LongCoT-Mini baselines (from external benchmarking, post-paper):
  GPT-5.2: ~38.7% on mini (VERIFY - sourced from search result, not paper directly)
  Claude Sonnet 4.5 vanilla: 2.6% (13/500) - measured by Weitekamp Apr 2026, not paper

KEY FINDING (paper claim, MEASURED): accuracy falls sharply as problem DAG size grows.
Individual step tractability is confirmed; the failure mode is error propagation over long chains.

RLM-AUGMENTED PERFORMANCE (NOT paper numbers - post-paper external measurements)
These numbers are from Weitekamp (raw.works) and X posts, NOT from the LongCoT paper itself:
  Claude Sonnet 4.5 + DSPy.RLM: 45.4% on Mini (vs 2.6% vanilla) - measured Apr 16, 2026
  Qwen3.5-9B + DSPy.RLM: ~15.69% on full (approx 1.6x GPT-5.2) - preliminary X post
  Qwen3.5-27B + DSPy.RLM: ~22.18% on full (approx 2x GPT-5.2) - preliminary X post
  Codex CLI + GPT-5.5 xhigh: 79.6% on Mini, 72.5% on full (1446/1995) - Weitekamp May 2026
  Opus 4.7 + DSPy.RLM: 75.4% on Mini (377/500) - Weitekamp May 2026
  WARN: Qwen3.5 numbers labeled "preliminary" by author; full benchmark slice unspecified

TASK-LEVEL RLM PERFORMANCE (Sonnet 4.5 + RLM on Mini, Weitekamp Apr 16, 2026):
  Perfect/near-perfect: Dungeon, Packaging, Hanoi, Wizards, TrapezoidCounting, Sudoku (15/15 each)
  Chess: 85/100 (vs 0/100 vanilla)
  BlocksWorld: 9/10 (vs 0/10); Sokoban: 7/10 (vs 0/10)
  Chemistry: 31/100 (vs 13/100) - modest gain
  Math: 6/95 - weak even with RLM
  Graph-structured: Hindley-Milner, MaxFlow-MinCut: 0/75 each - RLM fails here too

PAPER'S ORIGINAL CLAIM about compositional domains: "Chemistry, Math remain at zero"
STATUS: REFUTED by Weitekamp May 2026 (code execution harness: Math 69.5%, Chemistry 41.0%)
IMPLICATION: the "wall" is harness-dependent, not a model capability ceiling.

LIMITATIONS (paper-cited and observed)
- No detailed error taxonomy (only: failed, formatting, correct)
- Whether problems prevent pattern-matching is uncertain (not verified by paper)
- Long unbroken chain assumption may not match real-world tasks with subtasks/memory
- No distinction between reasoning-token budget exhaustion and reasoning error

FAILURE MODES
- Single error propagates forward and invalidates all subsequent steps (by design)
- Graph-structured dependency tasks (e.g. type inference, flow algorithms) resist tool augmentation
- Mathematics weak even with code execution tools in early results
- Cost: RLM costs ~20x vanilla ($621 vs $31 for 500 questions on Sonnet 4.5)

BENCHMARK PORTABILITY SUMMARY
OPEN: yes (MIT license, public GitHub + HF dataset)
SELF-CONTAINED: yes (problems are text + verifiable answers, no external oracle needed)
RUNNABLE: yes (uv sync + run_inference.py, multi-provider support)
REPRODUCIBLE: yes for primary metric (exact-match on verifiable answers)
GAPS: no built-in RLM configs (must add custom harness or use DSPy.RLM separately)
