# C-56: Codebase-Memory DKG 31-Repo Eval (Head-to-Head, RUNPOD-DEPENDENT)

Objective: run ELAI's DKG (Phase 1A + C-24 symbol_impact/symbol_flow/symbol_context
tools) head-to-head against the published Codebase-Memory baseline
(arXiv:2603.27277) on the paper's 31-repository evaluation set. Single direct
A/B against the published 83% answer quality / 10× token efficiency / 2.1×
tool-call efficiency.

Pre-requisites: B-29 SymbolGraphCache + B-30 multi-language SHIPPED;
C-38 (PageRank production contract) SHIPPED so query path is verified live;
RunPod online.
Parallel-safe with: C-54, C-55, C-57, C-58.

Paper grounding (arXiv:2603.27277, 2026-03-28, verified abs page 2026-05-17):
"Codebase-Memory: Tree-Sitter-Based Knowledge Graphs for LLM Code Exploration
via MCP." Measured baseline: "83% answer quality versus 92% for a
file-exploration agent, at ten times fewer tokens and 2.1 times fewer tool
calls" across 31 real-world repositories.

## Context

Codebase-Memory is the closest published architecture to ELAI's DKG +
plugins MCP bridge. The 31-repo evaluation is reproducible: source-grounding
via paper-ingestor confirmed open-source release.

ELAI's DKG should reproduce-or-beat 83% answer quality at SAME OR BETTER
token / tool-call efficiency. Two outcomes both useful:
- ELAI ≥ paper baseline: validates the architecture; publish.
- ELAI < paper baseline: identifies a measurable gap; the diff IS the
  improvement target (which DKG/CatRAG/A-RAG/reranker layer is weaker?).

## Reads

- F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-29_symbol_graph_cache.md
- F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-30_multi_language.md
- F:\.ELAI_workflow\docs\thefinalglance\C-plan\C-24_symbol_impact_dashboard_surface.md
- arXiv:2603.27277 + reference repo (fresh fetch)

## Tasks

1. **Source-grounding** — paper-ingestor (mode=paper + mode=source) against
   arXiv:2603.27277. Capture the 31-repo manifest, query set, answer-quality
   scoring rubric.

2. **31-repo corpus import** — vendor the 31 repos at pinned SHAs under
   .elai_cc/benchmarks/codebase_memory/repos/. Manifest in
   benchmarks/codebase_memory/manifest.toml.

3. **Query corpus** — the paper's evaluation set of N queries per repo,
   ground-truth answers. Pinned alongside.

4. **ELAI side adapter** — for each query, route through ELAI's DKG path:
   compute_repo_map → symbol_impact / symbol_flow / symbol_context as
   needed → emit answer. Record tokens consumed + tool calls used.

5. **Answer-quality scorer** — match paper's scoring rubric (likely
   model-as-judge or string-similarity per query type). Use Sonnet 4.6
   as judge for fairness; document the choice.

6. **Per-repo and aggregate metrics** — answer-quality %, mean tokens per
   query, mean tool calls per query. Cross-tabulate vs paper's published
   baseline.

7. **Hub-detection and caller-ranking specialized queries** — paper reports
   "matches or exceeds the explorer on 19 of 31 languages" on these
   specialized queries. ELAI's C-24 symbol_impact tool is a direct match
   for hub-detection — verify or beat.

8. **Methodology + results docs** — same structure as C-54.

9. **SSB action types** — codebase_memory_query_run, codebase_memory_answer_scored,
   codebase_memory_scorecard_emitted.

## WIRING_CONTRACT

produces_symbols:
  - 31-repo corpus + pinned SHAs
  - query corpus
  - CodebaseMemoryAdapter (crates/task_runner/src/cb_memory_adapter.rs)
  - AnswerQualityScorer (Sonnet 4.6 judge)
  - scorecard.json
  - methodology + results docs
  - SSB action_types: codebase_memory_query_run, codebase_memory_answer_scored,
    codebase_memory_scorecard_emitted
consumes_symbols:
  - DKG (Phase 1A) + PageRank (Phase 1D) + symbol tools (C-24)
  - Sonnet 4.6 backend (judge)
  - SSB event bus
reachable_from:
  - elai benchmark codebase-memory --runpod-id <id>
stub_ban:
  - judge model MUST be documented; not user-changeable per run
  - answer-quality scoring MUST be deterministic given fixed judge +
    fixed seed
  - per-repo results MUST be visible; not just aggregate
  - if ELAI < paper: results.md MUST surface the diff honestly with
    candidate-improvement targets (don't bury bad numbers)

## DETERMINISTIC_CHECK

```bash
ls .elai_cc/benchmarks/codebase_memory/repos | wc -l | awk '$1 == 31 {exit 0} {exit 1}'
test -f .elai_cc/benchmarks/codebase_memory/manifest.toml
test -f .elai_cc/crates/task_runner/src/cb_memory_adapter.rs
test -f workflow/sessions/$(cat workflow/sessions/current.txt)/cb_memory_run/scorecard.json
test -f .elai_cc/docs/codebase_memory_results.md
rg "codebase_memory_query_run|codebase_memory_answer_scored|codebase_memory_scorecard_emitted" .elai_cc/crates/orchestrator/src
```

## ACCEPTANCE

- [ ] paper-ingestor produces workflow/papers/codebase-memory-interface.md
- [ ] 31 repos vendored at pinned SHAs
- [ ] Query corpus + ground-truth answers vendored
- [ ] ELAI-side adapter implemented
- [ ] AnswerQualityScorer implemented with deterministic judge
- [ ] **31-repo run completed** — per-repo + aggregate metrics in
      scorecard.json
- [ ] Hub-detection / caller-ranking specialized queries scored separately
- [ ] All 3 SSB action types emit
- [ ] results.md authored with paper-baseline cross-tabulation
- [ ] If ELAI < paper baseline: improvement-target list authored in
      results.md (which layer to strengthen)
- [ ] Methodology doc generated

## Files

- ADD: .elai_cc/benchmarks/codebase_memory/repos/ (31 pinned repos)
- ADD: .elai_cc/benchmarks/codebase_memory/manifest.toml
- ADD: .elai_cc/benchmarks/codebase_memory/queries.json
- ADD: .elai_cc/crates/task_runner/src/cb_memory_adapter.rs
- ADD: .elai_cc/crates/task_runner/src/answer_quality_scorer.rs
- ADD: workflow/sessions/{id}/cb_memory_run/scorecard.json (post-run)
- ADD: workflow/sessions/{id}/cb_memory_methodology.md (post-run)
- ADD: .elai_cc/docs/codebase_memory_results.md (post-run)
- ADD: workflow/papers/codebase-memory-interface.md
- MODIFY: .elai_cc/crates/cli/src/main.rs (benchmark codebase-memory subcommand)
- MODIFY: .elai_cc/crates/orchestrator/src/ssb/schema.rs (3 new action types)

## Next

C-56 closure is the single most-direct architectural validation in C-35..C-59.
If ELAI matches or beats paper's 83% / 10× / 2.1× numbers, the DKG layer
ships on solid ground. If not, the diff is the improvement target for
the next planning cycle.
