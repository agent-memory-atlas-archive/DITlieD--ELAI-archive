# D-24: Cross-domain canary regression validation

Objective: 100 synthetic mutations (50 helpful, 30 neutral, 20 harmful)
across 5 task families; verify D-7 canary blocks all 20 harmful + at most
5 helpful (FP ≤ 10%) + ≤ 2 neutral.
Estimated effort: 1 session.
Pre-requisites: D-7 (canary implementation).
Parallel-safe with: D-19..D-25.

## Context

D-7 ships the canary; D-24 measures its discrimination quantitatively.

Per-mutation type:
- **Helpful**: target-family improves +5..+15%, no non-target regresses.
- **Neutral**: target-family unchanged ±2%, no non-target regresses.
- **Harmful**: target-family unchanged or improves, but ≥1 non-target
  regresses >5% (the case canary must block).

## Reads

- F:\.ELAI_workflow\docs\specs\guide\reasoning-improvement-grounded-2026-05-19.md
  (§3.5 canary — lines 268-276)
- .elai_cc/crates/orchestrator/src/canary.rs (D-7)

## Tasks

1. **Synthetic mutation fixtures** at .elai_cc/benchmarks/canary/fixtures/:
   - 100 fixtures: { mutation_id, target_family, baseline_pass_rates_by_family,
     candidate_pass_rates_by_family, expected_verdict }.
   - 50 helpful, 30 neutral, 20 harmful.
   - Each TOML.

2. **CanaryBenchmark** in
   .elai_cc/crates/orchestrator/src/benchmarks/canary_runner.rs:
   - Run CanaryRunner (D-7) on each fixture; compare to expected_verdict.
   - Report: total, true_block, false_block, true_pass, false_pass.

3. **Gate**: 20/20 harmful blocked; ≤5/50 helpful blocked; ≤2/30 neutral
   blocked.

4. **CLI `elai bench canary-regression`**.

5. **Decision doc** workflow/reasoning/D-24-decisions.md (3 decisions):
   - (a) Why 100 fixtures: enough for sample noise floor.
   - (b) Why 50/30/20 split: harmful is minority case; over-sampling distorts.
   - (c) Why ≤5 helpful FP: 10% FP gives room for tuning regression_threshold
     in D-7 without retraining canary infra.

## WIRING_CONTRACT

produces_symbols:
  - CanaryBenchmark + CanaryBenchmarkReport
  - `elai bench canary-regression` CLI
consumes_symbols:
  - CanaryRunner (D-7)
  - SsbWriter
reachable_from:
  - CLI + CI workflow
stub_ban:
  - 20/20 harmful blocked is hard gate
observable_effect:
  trigger: `elai bench canary-regression`
  expect_ssb: BenchmarkStarted + BenchmarkResult
  expect_artifact: report at .elai_cc/benchmarks/canary/reports/

## DETERMINISTIC_CHECK

```bash
ls .elai_cc/benchmarks/canary/fixtures/*.toml | wc -l
# Expected: 100
rg "struct CanaryBenchmark" .elai_cc/crates/orchestrator/src/benchmarks/
cargo check --workspace 2>&1 | tail -3
```

## ACCEPTANCE

- [ ] CanaryBenchmark compiles
- [ ] 100 fixtures present
- [ ] `elai bench canary-regression` exit 0 if gates met
- [ ] cargo check --workspace: zero new errors
- [ ] Cloud smoke test A: 20/20 harmful blocked
- [ ] Cloud smoke test B: ≤5/50 helpful blocked (FP ≤ 10%)
- [ ] Cloud smoke test C: ≤2/30 neutral blocked
- [ ] Decision doc workflow/reasoning/D-24-decisions.md

## Files touched

- ADD: .elai_cc/crates/orchestrator/src/benchmarks/canary_runner.rs
- ADD: .elai_cc/crates/cli/src/bench_canary.rs
- ADD: .elai_cc/benchmarks/canary/fixtures/{mutation_id}.toml × 100
- MODIFY: .elai_cc/crates/cli/src/commands.rs
- ADD: workflow/reasoning/D-24-decisions.md

## Out-of-scope

- Real-data mutations (DEFERRED; synthetic suffices for D-7 gate)

## Next

D-25 (determinism + archive tamper test suite).
