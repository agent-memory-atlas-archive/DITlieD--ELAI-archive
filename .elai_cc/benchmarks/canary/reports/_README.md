D-24 — Cross-Domain Canary Regression Benchmark Reports

This directory receives one `{ts_ms}.json` file per `elai bench
canary-regression --output <path>` run when the caller passes a path
inside this directory. The CLI driver defaults stdout when no `--output`
is supplied; CI workflows write to `/tmp/canary-bench-report.json` so
this directory stays clean in shared CI agents.

Report schema mirrors
`elai_orchestrator::benchmarks::canary_runner::CanaryBenchmarkReport`
(serde-derived JSON):

  true_block: u32
  false_block: u32
  true_pass: u32
  false_pass: u32
  harmful_blocked: u32
  helpful_blocked: u32
  neutral_blocked: u32
  total_fixtures: u32
  verdict: "pass" | "fail"
  fixture_blake3: 64-hex blake3 of the canonicalized fixture corpus
  provenance_counts: BTreeMap<String, u32>
  per_family_breakdown: BTreeMap<String, CanaryFamilyBreakdown>

The SSB row `CanaryBenchRunCompleted` carries the same aggregate
counters; per-family + per-provenance breakdowns live ONLY in the
file-based report (dashboard / CI consumers should read from disk
when they need the drill-down).
