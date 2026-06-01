---
name: benchmark_run
type: application
version: 1.0.0
description: Run a coding benchmark suite such as SWE-bench, polyglot, or HumanEval.
triggers: benchmark, swe-bench, polyglot, humaneval, run bench, measure, eval suite, leaderboard, score the model, bench
inputs: bench_name, model_target
outputs: bench_report, ssb_metric_rows
scope: evaluation
tags: [benchmark, evaluation, metric, batch, long-running]
estimated_loc: 0
estimated_minutes: 120
---

Application skill — type=application. Routes prompts asking to run a known
coding-benchmark suite to the orchestrator's batch executor.

Use when:
  - the user names a benchmark (SWE-bench, polyglot, HumanEval, MBPP, etc.)
  - the run produces SSB metric rows for dashboard consumption
  - the model target is known (local sidecar, cloud Sonnet, etc.)
  - the run is long-lived and benefits from a per-suite progress feed

Do NOT use for ad-hoc evals (focused_edit + a one-off script suffices), for
sidecar smoke (sidecar-protocol harness skill), or for a single-prompt
inference test (simple_query).

Output: a per-task ssb row stream (`bench_task_started`, `bench_task_completed`)
plus a summary report under `workflow/sessions/<sid>/bench/<suite>-<ts>.md`
with pass-rate, per-task latency, and a rerun manifest for failures.
