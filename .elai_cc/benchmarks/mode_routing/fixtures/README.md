D-21 mode-routing coverage benchmark fixture set.

mode_routing_mini.jsonl
-----------------------

Hand-labeled JSONL fixture used by `elai bench mode-routing` and by the
in-tree smoke test (`crates/cli/tests/bench_mode_routing_smoke.rs`).
25 rows balanced across the 5 GroundingMode variants (5 rows per mode):

  constructive                 5 rows
  falsifiability_cross_family  5 rows
  conformal_refusal            5 rows
  operational_deferred         5 rows
  conservative_fallback        5 rows

Each row is engineered so D-10's pure predicate
`crates/orchestrator/src/grounding_mode.rs::select_grounding_mode`
deterministically returns the row's `expected_mode` given the row's
context flags. This gives the runner a 100% baseline against the mini
fixture so dashboard / CI regressions surface immediately.

Per the D-21 plan (D-21_mode_routing_coverage_benchmark.md task 1 +
stub_ban): rows are hand-authored, not LLM-generated. The 200-row
full benchmark (40 per mode) is deferred per plan §Out-of-scope and
will land alongside the cloud test A live run.

Schema
------

Field set documented at
`crates/orchestrator/src/benchmarks/mode_routing_runner.rs::FixtureRow`.

Lines beginning with `#` and blank lines are skipped by the loader.
