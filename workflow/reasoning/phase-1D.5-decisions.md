Phase 1D.5 MEASURE decision doc

What: collect 7 metrics for the PageRank module and write .elai_cc/metrics/phase_1D.toml.
All values must come from auditor-run commands, not placeholders.

Key decision: measurement methodology
- test_count: parsed from cargo test --package knowledge -- pagerank output "test result:" line
- compile_time_seconds: parsed from cargo build --package knowledge --timings "Finished" line
- pagerank_loc: wc -l on the module file (914 lines per context, but auditor re-measures)
- ranking_quality: boolean derived from A2 + T3 test pass status
- token_budget_accuracy: float percentage from T6 + P3 pass (max deviation <= 15%)
- density_guard_correct: boolean derived from T4 + A1 test pass status
- binary_search_max_iterations: integer from T10 test (fit_iterations <= 10)

Expected values (approximate):
- test_count: 24 (confirmed by prior commits)
- compile_time_seconds: varies by machine, typically 2-10s
- pagerank_loc: ~914
- ranking_quality: true (all tests pass)
- token_budget_accuracy: <= 15.0
- density_guard_correct: true (all tests pass)
- binary_search_max_iterations: <= 10

Unknowns:
- Exact compile_time_seconds depends on machine load
- Exact token_budget_accuracy deviation percentage depends on test output detail
- If T6/P3 pass with 15% tolerance, record 15.0 as upper bound if exact value not extractable
