# D-19: PRMBench adversarial gaming benchmark for PPM

Objective: pull PRMBench fixtures (Deception Resistance + Circular Logic
subsets), run D-2 PPM against adversarial samples, report per-category score,
gate PPM promotion on ≥ conservative published baseline. PRMBench (arxiv:
2501.03124, T1) §4.2: 25 PRMs evaluated; worst categories are Deception
Resistance + Circular Logic — exactly what a gaming self-improver would
exploit. D-19 measures ELAI's exposure honestly.
Estimated effort: 2 sessions.
Pre-requisites: D-2 (PpmClient + PpmAdapter), D-3 (consensus rule consumes
  PPM score). RunPod NOT required (single-PPM eval; runs on remote sidecar).
Parallel-safe with: D-20..D-25 (benchmark slots isolated).

## Context

Companion §1.13 (lines 117-126). PRMBench three findings:
1. Deception Resistance + Circular Logic are worst-scoring categories
   (60-65% baseline for top PRMs).
2. MC-estimation PRMs underperform LLM-as-judge (arxiv:2501.07301, T1).
3. ThinkPRM (arxiv:2504.16828, T1) — generative verifier with verification
   CoT beats discriminative PRMs at 1% of labels on natural OOD; but does
   NOT evaluate adversarial gaming.

D-19 measures D-2 PPM (Qwen2.5-Math-PRM-7B per D-2) on PRMBench public
subsets. Reports honestly — if PPM scores < published baseline on adversarial
categories, that's evidence to either (a) accept the limitation (with D-3
consensus rule as defense), (b) DEFER promotion until D-28 adversarial
example farming improves PPM.

Partial foundation present and usable:
- D-2 PpmClient + PpmAdapter.
- D-3 VerificationResult.

## Reads

- F:\.ELAI_workflow\docs\specs\guide\reasoning-improvement-grounded-2026-05-19.md
  (§1.13 PRMBench — lines 117-126)
- F:\.ELAI_workflow\workflow\papers\prmbench-2501.03124-interface.md (D-2)
- .elai_cc/crates/orchestrator/src/ppm_client.rs (D-2)

## Paper-ingestor pre-step

Spawn paper-ingestor with mode=paper for:
- arxiv:2501.07301 Lessons of Developing PRMs
- arxiv:2504.16828 ThinkPRM

Verify workflow/papers/{prm-lessons-2501.07301,thinkprm-2504.16828}-interface.md.

## Tasks

1. **PRMBench fixture ingestion** at .elai_cc/benchmarks/prmbench/:
   - Fetch public PRMBench dataset subsets (HuggingFace dataset; check
     license; PRMBench is open).
   - Filter to Deception Resistance + Circular Logic subsets initially
     (additional categories DEFERRED).
   - Store as fixtures/{category}/{task_id}.json with fields:
     problem_statement, step_history, candidate_step, ground_truth_label
     (correct / incorrect).
   - Bootstrap 100 tasks per category (200 total).

2. **PRMBenchRunner** in new file
   .elai_cc/crates/orchestrator/src/benchmarks/prmbench_runner.rs:
   - `struct PRMBenchRunner { ppm: Arc<PpmClient>, fixtures_dir: PathBuf,
     output_dir: PathBuf }`
   - `struct PRMBenchReport { category: String, total_tasks: u32,
     correct_predictions: u32, false_positives: u32, false_negatives: u32,
     accuracy: f32, precision: f32, recall: f32, baseline: f32,
     verdict: GatingVerdict }`
   - `enum GatingVerdict { Pass, FailBelowBaseline, InsufficientData }`
   - `fn run_category(&self, category: &str) -> PRMBenchReport`:
     - Walk fixtures/{category}; for each task, call ppm.score_step;
       compare to ground_truth_label; update counters.
     - Compute accuracy / precision / recall.
     - Compare to published baseline (Qwen2.5-Math-PRM-7B reported ~60%
       Deception Resistance; D-19 uses conservative 35% gate per D-SUMMARY).

3. **CLI subcommand `elai bench prmbench`** in
   .elai_cc/crates/cli/src/bench_prmbench.rs:
   - Run PRMBenchRunner on all categories; emit report; exit 0 on Pass,
     1 on FailBelowBaseline.

4. **SSB action types** in ssb.rs (SCHEMA_VERSION 64→65):
   - `BenchmarkStarted { name: String, fixture_count: u32, audit_id: Uuid }`
   - `BenchmarkResult { name: String, accuracy: f64, baseline: f64,
     verdict: String, audit_id: Uuid }`
   - Round-trip tests (reused across D-19..D-25).

5. **CI workflow** at .elai_cc/.github/workflows/bench-prmbench.yml:
   - Triggered on push + PR + weekly cron.
   - Steps: checkout → boot PPM sidecar (requires SSH access; documented as
     manual prereq if remote unavailable) → `elai bench prmbench` →
     publish report as CI artifact.
   - On FailBelowBaseline: warn (CI continues; does NOT block merge) until
     PPM is in production-gating role.

6. **Decision doc** workflow/reasoning/D-19-decisions.md (3 decisions):
    - (a) Why conservative ≥35% (not published 60-65%): allow margin for
      Qwen2.5-Math-PRM-7B base model variance; gate ELAI use, not paper
      reproduction.
    - (b) Why Deception + Circular Logic only initially: highest-risk
      categories per PRMBench §4.2; other categories DEFERRED.
    - (c) Why CI warn (not block) on fail: PPM is verifier consensus signal
      (D-3); single-signal failure ≠ blocking; D-3 consensus rule defends.

## WIRING_CONTRACT

produces_symbols:
  - PRMBenchRunner + PRMBenchReport + GatingVerdict
  - run_category + CLI `elai bench prmbench`
  - SsbActionType::BenchmarkStarted + ::BenchmarkResult + payloads
  - SCHEMA_VERSION bump 64→65
  - .elai_cc/benchmarks/prmbench/fixtures/ + reports/ layout
  - bench-prmbench.yml CI workflow
consumes_symbols:
  - PpmClient (D-2)
  - SsbWriter
reachable_from:
  - `elai bench prmbench` CLI
  - bench-prmbench.yml CI step
stub_ban:
  - run_category MUST handle PPM timeout (mark task InsufficientData, continue)
  - report MUST include baseline + verdict (no silent pass)
  - 35% threshold MUST load from elai_cc.toml or args, not hardcoded
observable_effect:
  trigger: `elai bench prmbench --category=deception_resistance`
  expect_ssb: 1 BenchmarkStarted + 1 BenchmarkResult row
  expect_artifact: .elai_cc/benchmarks/prmbench/reports/{date}-{category}.json

## DETERMINISTIC_CHECK

```bash
rg "SCHEMA_VERSION: i64 = 65" .elai_cc/crates/orchestrator/src/ssb.rs
rg "BenchmarkStarted|BenchmarkResult" .elai_cc/crates/orchestrator/src/ssb.rs
rg "struct PRMBenchRunner|fn run_category" .elai_cc/crates/orchestrator/src/benchmarks/prmbench_runner.rs
ls .elai_cc/benchmarks/prmbench/fixtures/*/ | wc -l
# Expected: >= 200 (100 per category × 2 categories)
test -f .elai_cc/.github/workflows/bench-prmbench.yml
cargo check --workspace 2>&1 | tail -3
cargo test -p elai_orchestrator -- prmbench 2>&1 | grep "test result"
test -f workflow/papers/prm-lessons-2501.07301-interface.md && \
  test -f workflow/papers/thinkprm-2504.16828-interface.md && echo OK
```

## ACCEPTANCE

- [ ] SCHEMA_VERSION_PUBLIC == 65
- [ ] 2 SSB variants round-trip
- [ ] PRMBenchRunner compiles; unit tests pass
- [ ] 100+ fixtures per category present (200 total minimum)
- [ ] `elai bench prmbench` CLI runs successfully
- [ ] bench-prmbench.yml CI workflow present
- [ ] cargo check --workspace: zero new errors
- [ ] cargo test -p elai_orchestrator -- prmbench: all pass, ≥6 tests
- [ ] Cloud smoke test A: live PPM (D-2 sidecar online); run prmbench on
      Deception Resistance subset; report scoring; verdict Pass if accuracy
      ≥35% (D-19 conservative gate); FailBelowBaseline otherwise (informational)
- [ ] Cloud smoke test B: same on Circular Logic subset; accuracy ≥30% gate
- [ ] Report files persisted at .elai_cc/benchmarks/prmbench/reports/
- [ ] All paper-ingestor outputs present
- [ ] Decision doc workflow/reasoning/D-19-decisions.md

## Files touched

- ADD: .elai_cc/crates/orchestrator/src/benchmarks/prmbench_runner.rs
- ADD: .elai_cc/crates/cli/src/bench_prmbench.rs
- ADD: .elai_cc/benchmarks/prmbench/fixtures/{deception_resistance,
       circular_logic}/{task_id}.json × 200+
- MODIFY: .elai_cc/crates/orchestrator/src/ssb.rs (SCHEMA_VERSION 64→65;
    2 new variants)
- MODIFY: .elai_cc/crates/cli/src/commands.rs (SlashCommand::BenchPrmbench)
- MODIFY: .elai_cc/crates/orchestrator/src/lib.rs (pub mod benchmarks; add
    submodule prmbench_runner)
- ADD: .elai_cc/.github/workflows/bench-prmbench.yml
- ADD: workflow/reasoning/D-19-decisions.md
- ADD: workflow/papers/{prm-lessons-2501.07301,thinkprm-2504.16828}-interface.md

## Out-of-scope

- Additional PRMBench categories (DEFERRED)
- ThinkPRM-style generative verifier integration (DEFERRED to D-28)
- Adversarial PPM retraining loop (D-28 DEFERRED)

## Next

D-20 (ARC-AGI-2 OOD probe) is next.
