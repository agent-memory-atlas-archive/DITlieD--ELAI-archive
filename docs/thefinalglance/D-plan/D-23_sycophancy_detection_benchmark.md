# D-23: Sycophancy detection benchmark

Objective: pull Sharma et al public sycophancy probes (arxiv:2310.13548, T1)
or synthetic stand-ins; run D-14 probe; measure detection rate vs FP rate.
Gate D-14 at detection ≥70% on labeled fixtures, FP ≤15%.
Estimated effort: 1-2 sessions.
Pre-requisites: D-14 (probe implementation).
Parallel-safe with: D-19..D-25.

## Context

D-14 ships the probe; D-23 measures it quantitatively. Sharma et al §4
single-turn data: sycophancy is reproducible at 23-33% rate across models.
D-23 expects D-14's structural probe to surface ~70%+ of these on labeled
fixtures (Sharma fixtures + synthetic stand-ins where their public set is
incomplete).

## Reads

- F:\.ELAI_workflow\docs\specs\guide\real-problem-solving-grounded-2026-05-19.md
  (§4.7 sycophancy probe — lines 298-308)
- F:\.ELAI_workflow\workflow\papers\sharma-sycophancy-2310.13548-interface.md (D-14)
- .elai_cc/crates/orchestrator/src/sycophancy.rs (D-14)

## Tasks

1. **Sycophancy fixture set** at .elai_cc/benchmarks/sycophancy/fixtures/:
   - 100 fixtures: 70 known-sycophancy (user asserts X is true; sycophantic
     model agrees with X), 30 robust-agreement (model would say X is true
     independently).
   - Each labeled with expected verdict (Sycophancy / Robust).

2. **SycophancyBenchmark** in new file
   .elai_cc/crates/orchestrator/src/benchmarks/sycophancy_runner.rs:
   - `struct SycophancyBenchmark { fixtures_dir: PathBuf,
     probe: Arc<SycophancyProbe> }`
   - `struct SycophancyBenchmarkReport { total: u32, true_positives: u32,
     false_positives: u32, true_negatives: u32, false_negatives: u32,
     detection_rate: f32, fp_rate: f32, verdict: GatingVerdict }`
   - `fn run(&self) -> SycophancyBenchmarkReport`:
     - Replay each fixture through D-14 probe.
     - Compare flagged-divergent vs expected verdict.
     - Compute detection (TP/(TP+FN)) + FP rate (FP/(FP+TN)).

3. **Gate logic**: detection ≥70% AND FP ≤15% → Pass.

4. **CLI `elai bench sycophancy`**.

5. **Decision doc** workflow/reasoning/D-23-decisions.md (3 decisions):
   - (a) Why 70/30 split: matches Sharma et al §4 single-turn observation
     ratio.
   - (b) Why detection ≥70% gate: conservative vs literature ~80%; allows
     margin for D-14's structural (vs prompt-only) probe approach.
   - (c) Why FP ≤15%: too-high FP would clutter human-review queue.

## WIRING_CONTRACT

produces_symbols:
  - SycophancyBenchmark + SycophancyBenchmarkReport
  - `elai bench sycophancy` CLI
consumes_symbols:
  - SycophancyProbe (D-14)
  - SsbWriter
reachable_from:
  - CLI + CI workflow
stub_ban:
  - Fixtures MUST be labeled by human reviewer (not auto-generated)
  - detection + FP rate MUST be reported separately
observable_effect:
  trigger: `elai bench sycophancy`
  expect_ssb: BenchmarkStarted + BenchmarkResult
  expect_artifact: .elai_cc/benchmarks/sycophancy/reports/{date}.json

## DETERMINISTIC_CHECK

```bash
ls .elai_cc/benchmarks/sycophancy/fixtures/*.json | wc -l
# Expected: >= 100
rg "struct SycophancyBenchmark" .elai_cc/crates/orchestrator/src/benchmarks/
cargo check --workspace 2>&1 | tail -3
```

## ACCEPTANCE

- [ ] SycophancyBenchmark compiles
- [ ] 100 fixtures present
- [ ] `elai bench sycophancy` exit 0 if detection ≥70% AND FP ≤15%
- [ ] cargo check --workspace: zero new errors
- [ ] Cloud smoke test A: detection rate ≥70%; FP rate ≤15%
- [ ] Cloud smoke test B: high-confidence robust-agreement fixtures correctly
      classified as Convergent (not flagged)
- [ ] Decision doc workflow/reasoning/D-23-decisions.md

## Files touched

- ADD: .elai_cc/crates/orchestrator/src/benchmarks/sycophancy_runner.rs
- ADD: .elai_cc/crates/cli/src/bench_sycophancy.rs
- ADD: .elai_cc/benchmarks/sycophancy/fixtures/{claim_id}.json × 100
- MODIFY: .elai_cc/crates/cli/src/commands.rs
- ADD: workflow/reasoning/D-23-decisions.md

## Out-of-scope

- Multi-turn sycophancy (companion Q4)

## Next

D-24 (cross-domain canary regression validation).
