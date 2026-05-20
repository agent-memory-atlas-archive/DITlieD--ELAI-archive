D-22 — Conformal calibration validation fixtures

Layout
- fixtures/conformal_validation_mini.jsonl
    Mini balanced fixture (36 rows, 12 per distribution bucket) used by
    `elai bench conformal` against a balanced 20-pair calibration corpus.
    Each row carries:
      row_id           stable per-row id
      distribution     in_domain | near_ood | far_ood
      domain           code | math | prose | mixed | planning | tool_use
      predicted_score  composite confidence in [0.0, 1.0]
      ground_truth     post-hoc verdict (true | false)

Purpose
The runner (`ConformalValidationRunner` in
`crates/orchestrator/src/benchmarks/conformal_validation_runner.rs`)
loads the fixture, computes tau from a calibration corpus, splits the
fixture into the 3 distribution buckets via the `distribution` field,
runs `decide()` per row, and reports empirical refusal rate per bucket.
The verdict passes iff every bucket lands within
`[target - epsilon, target + epsilon]` AND every bucket has at least
DEFAULT_MIN_ROWS_PER_DISTRIBUTION rows (no class collapse).

Grounding
- arXiv:2503.01345 (UniCR) — formal distribution-free guarantee the
  runner validates.
- Vovk, Gammerman, Shafer — Algorithmic Learning in a Random World §2.2
  (quantile-based conformal predictor).
- docs/thefinalglance/D-plan/D-22_conformal_calibration_validation.md
  for the plan, WIRING_CONTRACT, and ACCEPTANCE checklist.

The mini fixture is engineered so the balanced 20-pair corpus at
epsilon=0.1 yields tau≈0.8 → `decide()` Abstains only for rows with
`predicted_score < 0.2`. One low-score row per bucket → empirical
refusal rate ≈ 1/12 ≈ 0.083, well inside the [0, 0.2] target band.

CI usage
The `elai bench conformal` subcommand reads this fixture by default,
runs the benchmark, emits one `conformal_validation_completed` SSB
row, writes the JSON report to `--output` (default stdout), and exits
0 on Pass / 1 on Fail.
