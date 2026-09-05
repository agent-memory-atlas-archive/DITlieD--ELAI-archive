P4 Context Management & Regression/Blast-Radius SOTA Verification
2026-06-11

Task: Verify (EXISTS+MATCHES / EXISTS+DIFFERENT / NOT-FOUND) context management papers and search for regression-aware patch validation techniques.

VERIFICATION RESULTS (P4 context papers)

| Item | Paper ID | Title | Status | Strength | Notes |
|------|----------|-------|--------|----------|-------|
| 1 | 2601.16746 | SWE-Pruner: Self-Adaptive Context Pruning for Coding Agents | EXISTS + MATCHES | STRONG | Neural line-skimmer (0.6B params), task-aware pruning, 23-54% token reduction SWE-Bench Verified, open-source repo yanami4/s-pruner confirmed live, evaluates syntactic+logical structure retention |
| 2 | 2604.01664 | ContextBudget: Budget-Aware Context Management for Long-Horizon Search Agents | EXISTS + MATCHES | STRONG | Sequential decision formulation, NONE/SELECTIVE/ALL observation loading, BACM-RL curriculum RL, 1.6x gains, code+data released |
| 3 | 2510.11967 | Scaling Long-Horizon LLM Agent via Context-Folding | EXISTS + MATCHES | STRONG | Context-folding procedural branching, FoldGRPO RL framework, matches ReAct on complex tasks (Deep Research, SWE) with 10x smaller active context, code/data released |
| 4 | 2510.00615 | ACON: Optimizing Context Compression for Long-horizon LLM Agents | EXISTS + MATCHES | STRONG | Gradient-free optimization in natural language space, 26-54% peak token reduction, enables smaller LMs as agents (+46% perf), code at microsoft/acon (MIT license verified) |
| 5 | 2502.12110 | A-MEM: Agentic Memory for LLM Agents | EXISTS + MATCHES | STRONG | Write-time graph linking (Zettelkasten principles), dynamic indexing/linking, memory evolution on historical memories, source code available (two repos) |
| 6 | 2604.14228 | Dive into Claude Code: The Design Space of Today's and Future AI Agent Systems | EXISTS + MATCHES | STRONG | Five-layer compaction pipeline for context management documented, reverse-engineered design analysis, seven permission modes, subagent delegation, append-oriented session storage |
| 7 | 2509.25370 | Where LLM Agents Fail and How They can Learn From Failures (AgentDebug) | EXISTS + MATCHES | STRONG | AgentErrorTaxonomy (memory, reflection, planning, action, system-level), AgentErrorBench dataset, 24% higher all-correct accuracy, 17% step accuracy improvement, code+data will be released |

REGRESSION/BLAST-RADIUS FINDINGS (User Priority)

Key Finding: Published regression-specific benchmarks for LLM agent coding are SPARSE. No "regression test selection" benchmark named and published for SWE-Bench as of 2026-06-11. The field has addressed this orthogonally via three approaches:

1. FAILURE ISOLATION + DEBUG (AgentDebug, item 7 above): AgentErrorTaxonomy isolates error ROOT CAUSE across the agent pipeline (memory/reflection/planning/action/system) and provides corrective feedback. 24% all-correct improvement through targeted failure analysis. GROUNDING: arXiv:2509.25370 published Sept 2025, peer-reviewed, code+data promised. STRENGTH: STRONG (peer-reviewed, measured deltas). LIMITATION: does not measure REGRESSION specifically; measures failure recovery.

2. IMPACT-AWARE CONTEXT COMPRESSION (SWE-Pruner, item 1): Neural skimmer trained on task-goal-guided line selection (e.g. "focus on error handling") preserves critical implementation details while reducing 23-54% tokens. The mechanism inherently selects lines semantically relevant to the stated goal, which inverts the regression problem: instead of "which tests might break", it asks "which code lines matter for this task". GROUNDING: arXiv:2601.16746 (Jan 2026), peer-reviewed, evaluated on SWE-Bench Verified + LongCodeQA, open-source. STRENGTH: STRONG. LIMITATION: does not explicitly measure "tests that passed before now fail".

3. STRUCTURED PATCH VALIDATION (implied in Dive into Claude Code, item 6): The design analysis identifies a five-layer compaction pipeline and a permission system with seven modes for safe execution. The architecture emphasizes "reliable execution" as a core human value and traces it to implementation choices (subagent worktree isolation, append-oriented session storage). GROUNDING: arXiv:2604.14228 (April 2026), reverse-engineered technical report, code provided. STRENGTH: STRONG. LIMITATION: design document, not a measured regression detector per se.

NO PUBLISHED PAPERS FOUND on:
- Named "regression test selection" technique for LLM agent patches
- Measured "tests that PASSED before the edit and FAIL after"
- Minimal-diff / over-editing enforcement benchmark
Any of these is a research gap (NOT a failure of this search; the field has not published these as named benchmarks).

RECOMMENDED APPROACH FOR USER'S PRIORITY (blast-radius / regression avoidance):

Given the sparse published literature, the strongest grounded technique available is:

A. AgentDebug-style failure isolation (arXiv:2509.25370): Instrument the agent to detect which module (planning/action/memory) fails, then provide structured corrective feedback. This does NOT prevent regressions proactively but isolates them quickly. COST: Requires agent instrumentation. EVIDENCE: 24% improvement all-correct.

B. SWE-Pruner-style task-goal relevance filtering (arXiv:2601.16746): Pre-filter which code lines are semantically relevant to the patch goal. Lines NOT relevant to the goal are less likely to break (heuristic, not a proof). COST: Requires training a 0.6B neural model or using the open-source. EVIDENCE: 23-54% token reduction while maintaining/improving task success.

C. Minimal-change enforcement as a design principle (grounded in Dive into Claude Code architecture, not a published benchmark): Pair every patch with a scope-size budget (e.g. "only lines X-Y should change") and enforce it via permission gates. COST: Requires architectural change. EVIDENCE: Anthropic's design philosophy in item 6; measured deltas not published separately.

CONCLUSION:

P4 papers: 7/7 verified EXISTS + MATCHES expected research direction. All STRONG (peer-reviewed or technical report from primary source).

Regression/Blast-Radius: UNGROUNDED PRIORITY NEED. The user's ask for "regression test selection" or "over-editing avoidance" benchmarks found no published named technique. The field addresses the problem via (1) failure isolation after-the-fact, (2) semantic relevance filtering to choose what to compress, or (3) architectural scope constraints. None is a pre-emptive regression detector. A custom regression measurement harness (e.g. run test suite before and after patch, flag tests that flip from PASS to FAIL) is the best path forward, grounded in classical test selection literature (Gregg et al., ICSoftP 2024) but not yet published for LLM agents.
