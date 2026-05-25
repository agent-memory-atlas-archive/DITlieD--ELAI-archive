D-22 — Conformal validation 300-row smoke corpus

Layout
- low_risk.jsonl   100 rows, predicted_score in [0.80, 0.99] (high-confidence Answer)
- boundary.jsonl   100 rows, predicted_score in [0.10, 0.30] (near-tau, mixed Abstain/Answer)
- high_risk.jsonl  100 rows, predicted_score in [0.02, 0.19] (low-confidence Abstain)

Each row matches the canonical `ValidationFixtureRow` shape consumed by
`crates/orchestrator/src/benchmarks/conformal_validation_runner.rs`:

  row_id           stable per-row id (`<bucket>_<3-digit-idx>`)
  distribution     in_domain | near_ood | far_ood (closed `DistributionBucket` set)
  domain           code | math | prose | mixed | planning | tool_use (closed `Domain` set)
  predicted_score  composite confidence in [0.0, 1.0]
  ground_truth     post-hoc verdict (true | false)

Purpose
This is the D-22 closure smoke set — a deterministic, schema-compliant synthetic
benchmark fixture. Mirrors the D-19 PRMBench fixture pattern: pure synthetic
data, not production traffic, used solely for CI/operator smoke runs against
the conformal-validation gating path.

Bucket sizing
Every distribution bucket carries ≥ DEFAULT_MIN_ROWS_PER_DISTRIBUTION (12)
rows so the runner's class-collapse guard never fires on this fixture set.

  low_risk:   40 in_domain, 35 near_ood, 25 far_ood
  high_risk:  30 in_domain, 35 near_ood, 35 far_ood
  boundary:   35 in_domain, 35 near_ood, 30 far_ood

CalibrationCorpus drift note (DECISION-3 of workflow/reasoning/D-22-decisions.md)
The D-22 closure task brief lists a per-row schema of
`{claim_score: f64, ground_truth_correct: bool, source: String}` — that is the
CalibrationCorpus tuple shape (which lives under `.elai_cc/calibration/seed/`,
not under `.elai_cc/benchmarks/`). The shipped runner consumes the
ValidationFixtureRow shape documented above; these fixtures match the runner,
not the task brief's CalibrationCorpus mention. Documented in
workflow/reasoning/D-22-decisions.md DECISION-3.

Grounding
- arXiv:2509.01455 (UniCR Eq.5) — quantile-based CRC formula the runner gates
  against; canonical paper-of-record per D-16 ingestion. The runtime code's
  inline `arXiv:2503.01345` citation is a cite-drift artefact (DECISION-7).
- Vovk, Gammerman, Shafer — Algorithmic Learning in a Random World §2.2
  (quantile-based conformal predictor closed form).
- docs/thefinalglance/D-plan/D-22_conformal_calibration_validation.md
  for the plan, WIRING_CONTRACT, and ACCEPTANCE checklist.

CI usage
The `elai bench conformal` subcommand (NOT plan-named `conformal-validate` —
drift documented in DECISION-4) reads any of these fixtures via `--dataset`:

  elai bench conformal --dataset benchmarks/conformal_validation/fixtures/low_risk.jsonl
  elai bench conformal --dataset benchmarks/conformal_validation/fixtures/boundary.jsonl
  elai bench conformal --dataset benchmarks/conformal_validation/fixtures/high_risk.jsonl

Each invocation runs the benchmark against the synthesised balanced 20-pair
calibration corpus (tau ≈ 0.8), writes one `conformal_validation_completed`
SSB row, renders the JSON report, and exits 0 on Pass / 1 on Fail.
