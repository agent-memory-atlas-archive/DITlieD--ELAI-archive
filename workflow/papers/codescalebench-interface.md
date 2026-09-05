Source: CodeScaleBench: Testing coding agents on large codebases and multi-repo software engineering tasks
URL (blog): https://sourcegraph.com/blog/codescalebench-testing-coding-agents-on-large-codebases-and-multi-repo-software-engineering-tasks
URL (GitHub): https://github.com/sourcegraph/CodeScaleBench
URL (ebook landing): https://sourcegraph.com/resources/ebooks/code-scale-bench-report
Author: Stephanie Jarmak (blog post)
Published: 3 March 2026
Organization: Sourcegraph
Reliability: MEDIUM (vendor blog + open GitHub repo; numbers confirmed across 2 independent fetches of blog
and GitHub README; NOT peer-reviewed; self-reported by Sourcegraph whose MCP is the intervention being tested;
treat as directional evidence, not ground truth)

CORE FINDING
Sourcegraph MCP navigation tools (13 tools: semantic search, symbol resolution, dependency tracing, cross-repo nav
and others) substantially improve large-codebase file retrieval: file recall 0.127->0.277, Precision@5 0.140->0.478.
MCP-augmented agents are also 30% cheaper ($0.73->$0.51) and 38% faster per task.
Gains are largest on cross-repo/org-scale work (incident debugging +11.2%, security +10.6%, Understand +11.5%).

SETUP
Benchmark: CodeScaleBench (CSB). 370-371 tasks total, 2 divisions.
  CSB-SDLC: 150 tasks across 9 lifecycle phases (Understand, Design, Feature, Fix, Test, Document, Refactor, Secure, Debug)
  CSB-Org: 220 cross-repository tasks across 40+ repositories in 9 programming languages
GitHub README also lists 275 tasks across 20 suites (apparent discrepancy, likely different version or subset; blog=370).
Agent: Claude Code + Haiku 4.5 (same model, identical tasks, two conditions).
Baseline: full local source code + standard tools (grep, file, read). No MCP.
MCP condition: same agent + 13 Sourcegraph MCP tools.
Evaluation: file recall, Precision@5, F1@5, mean task reward (0-1 scale), cost per task (USD), wall-clock time.

NAVIGATION TOOLS TESTED
From GitHub README (confirmed enumerated list, 8 tools named explicitly):
  keyword_search       -- full-text code search
  nls_search           -- semantic / neural code search
  read_file            -- remote file access
  find_references      -- cross-reference detection
  go_to_definition     -- definition navigation
  search_code          -- GitHub API code search
  get_file_contents    -- GitHub file retrieval
  get_repository_tree  -- repository structure browsing

Blog post describes "13 Sourcegraph MCP tools" total. Additional tools mentioned but not enumerated:
  dependency tracing, cross-repo navigation (named in blog). Remaining 5 tool names not confirmed from public sources.
  The 8 above are the confirmed enumerated set from the GitHub README.

VERBATIM EXTRACTS (from blog + GitHub README)

Retrieval metrics (blog post, confirmed across 2 fetches):
"File recall: 0.127 → 0.277"
"Precision@5: 0.140 → 0.478"
"F1@5: 0.099 → 0.262"

CSB-Org only (blog post):
"Baseline P@5: 0.007, R@5: 0.002"
"MCP P@5: 0.471, R@5: 0.149"

Cost and speed (blog post):
"Cost per task: $0.73 → $0.51 (approximately 30% reduction)"
"Agent execution time improvement: 38% shorter with MCP"
"Wall-clock time reduction: 47 seconds average"

Task type gains (blog post):
"The most significant effects are on incident debugging (11.2% gain) and security (10.6% gain)"
"The largest SDLC improvement was 'Understand' at 11.5% gain"

On retrieval quality (blog post verbatim):
"MCP retrieval substantially improves the amount and quality of context retrieved compared to the baseline."

On cross-repo advantage (blog post verbatim):
"Sourcegraph MCP tools are optimized for large-scale cross-repo retrieval, returning targeted file paths and
symbol hits across repositories quickly and precisely."

Task distribution (GitHub README, confirmed):
Crossrepo: 47 tasks | Understand: 44 | Refactor: 43 | Security: 39 | Feature: 34
Debug: 26 | Fix: 19 | Test: 12 | Document: 11

Baseline mean reward (GitHub README): 0.536-0.565 across 371 mixed tasks with Claude Haiku 4.5.

NOTE on "9-task subset": the research report in scope references CSB-SDLC's 9 lifecycle phases as the "9-task subset,"
NOT 9 individual tasks. The 9 phases are the ones listed under SETUP above.

REIMPLEMENTATION NOTES FOR ELAI
ELAI's shipped symbol-graph navigation: impact_radius.py, call_flow (via codegraph MCP), symbol_context
(codegraph_explore/callers/callees), elai.map grep. These are the ELAI analog of the Sourcegraph MCP navigation tools.

This paper is PRO-NAVIGATION production evidence. It answers: does wiring symbol-graph navigation as live
agent actions improve large-codebase retrieval? Answer: yes, substantially (P@5 3.4x, file recall 2.2x).

Concrete ELAI implications:
1. The existing symbol-graph tools (impact_radius, codegraph MCP) are already shipped. This benchmark justifies
   wiring them as live agent navigation actions callable during retrieval, not just as pre-edit planning tools.
2. The gain is largest on cross-repo and org-scale tasks -- exactly the case where ELAI's elai.map / SCIP
   cross-language references would contribute most. The codegraph MCP (live, file-watched) covers call-chain
   traversal; elai.map covers SCIP-resolved canonical refs. The combination maps closely to
   "find_references + go_to_definition + dependency tracing" in the Sourcegraph tool set.
3. Cost argument is strong: MCP tools are NOT more expensive -- they are cheaper ($0.73->$0.51) because
   fewer wasted token rounds occur when the agent gets precise file paths immediately instead of grepping.
   This directly refutes the "symbol-graph tools are expensive" concern for ELAI's retrieval path.
4. The Sourcegraph tools are semantically similar to what ELAI already has; the missing piece is wiring them
   into the agent's live retrieval loop (cf. A7's RepoGraph finding about graph-based navigation improving
   SWE-bench). CodeScaleBench provides the production-scale corroboration.
5. Caveat: Sourcegraph is the vendor. The benchmark is self-reported. The numbers should be treated as
   directional. An independent reproduction on ELAI's own codebase tasks would be the grounded verification.

GROUNDED CHECKS (testable assertions against codescalebench numbers)
1. assert mcp_precision_at_5 >= 3.0 * baseline_precision_at_5, "MCP P@5 at least 3x baseline per blog (0.478 vs 0.140 = 3.4x)"
2. assert mcp_file_recall >= 2.0 * baseline_file_recall, "MCP file recall at least 2x baseline per blog (0.277 vs 0.127 = 2.2x)"
3. assert mcp_cost_per_task < baseline_cost_per_task, "MCP cheaper than baseline per blog ($0.51 < $0.73)"
4. assert mcp_exec_time_reduction_pct >= 35, "MCP at least 35% faster per blog (38%)"
5. assert 'find_references' in confirmed_tool_list and 'go_to_definition' in confirmed_tool_list, "symbol navigation tools core to the MCP per GitHub README"
