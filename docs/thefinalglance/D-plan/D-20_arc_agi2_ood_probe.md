# D-20: ARC-AGI-2 OOD probe harness — genuine-novelty gating

Objective: pull public ARC-AGI-2 subset (or synthetic ARC-style tasks if
hand-crafted leakage flagged); run ELAI with mode-routing on; report
solve-rate + Mode-E refusal-rate vs human baseline. Per ARC Prize 2025 Tech
Report (arxiv:2601.10904, T1): top score 24.03% (NVARC) vs 60-70% for
humans. Even hand-crafted novelty benchmarks leak (contamination warning §1).
D-20 measures ELAI honestly; no DiscoveryCandidate auto-published.
Estimated effort: 2-3 sessions. Cloud Sonnet only (no local 7B for ARC).
Pre-requisites: D-10 (mode routing), D-15 (novelty labeling).
Parallel-safe with: D-19, D-21..D-25.

## Context

Companion §1.16 (lines 143-148) + real-problem §1 Mode C (lines 76-89).
Honest framing:
- ARC-AGI-2 designed to resist memorization AND search.
- Even frontier models (Claude Opus, GPT-4o, DeepSeek-R1) trail dramatically.
- Tiny 7M-param recursive models reach 45% on ARC-AGI-1 → scale not the bottleneck.
- ELAI on cloud Sonnet ≠ frontier; expect single-digit to low-double-digit
  solve rate.

D-20 measures + reports. Pass criteria are HONEST: any solve-rate is fine;
the test gates on (a) Mode-E refusal-rate in healthy band (D-17 SLA), (b)
no DiscoveryCandidate auto-published (per D-15 design — human review only),
(c) sycophancy-flag rate negligible (D-14).

Partial foundation present and usable:
- D-10 mode routing.
- D-15 novelty labeling.
- D-17 refusal SLA.

## Reads

- F:\.ELAI_workflow\docs\specs\guide\reasoning-improvement-grounded-2026-05-19.md
  (§1.16 ARC-AGI-2 implication — lines 143-148)
- F:\.ELAI_workflow\docs\specs\guide\real-problem-solving-grounded-2026-05-19.md
  (§1 Mode C — lines 68-89)
- F:\.ELAI_workflow\workflow\papers\arc-prize-2025-2601.10904-interface.md (D-6)
- F:\.ELAI_workflow\workflow\papers\chollet-arc-1911.01547-interface.md (D-15)

## Paper-ingestor pre-step

(Both papers ingested in D-6 + D-15 — no new ingest required.)

## Tasks

1. **ARC-AGI-2 fixture ingestion** at .elai_cc/benchmarks/arc_agi2/fixtures/:
   - Pull public ARC-AGI-2 evaluation subset (50 tasks; arcprize.org provides).
   - Convert from JSON to typed structure: ArcTask { task_id, train_pairs:
     Vec<(InputGrid, OutputGrid)>, test_inputs: Vec<InputGrid>,
     test_outputs: Vec<OutputGrid> }.
   - Document version/checksum to detect dataset drift.

2. **ARCAGI2Runner** in new file
   .elai_cc/crates/orchestrator/src/benchmarks/arc_agi2_runner.rs:
   - `struct ARCAGI2Runner { fixtures_dir: PathBuf, output_dir: PathBuf,
     cloud_client: Arc<dyn CloudClient> }` (cloud Sonnet only, per ARC
     compute requirements)
   - `struct ARCReport { total_tasks: u32, solved: u32, refused: u32,
     mode_distribution: HashMap<GroundingMode, u32>, novelty_distribution:
     HashMap<NoveltyClass, u32>, sycophancy_flag_count: u32, solve_rate: f32,
     refusal_rate: f32, human_baseline: f32, verdict: ARCVerdict }`
   - `enum ARCVerdict { HonestlyEvaluated, RefusalRateOutOfBand,
     ContaminationSuspected }`
   - `fn run(&self) -> ARCReport`:
     - For each ArcTask, construct Claim; route through mode predicate;
       observe ClaimResult.
     - Solved = ClaimResult::Verified AND output matches expected.
     - Aggregate distributions.

3. **CLI subcommand `elai bench arc-agi2`**:
   - In .elai_cc/crates/cli/src/bench_arc.rs.
   - Runs ARCAGI2Runner; emits report.
   - Exit 0 if verdict == HonestlyEvaluated regardless of solve-rate;
     exit 1 if RefusalRateOutOfBand OR ContaminationSuspected.

4. **Contamination heuristic** in arc_agi2_runner.rs:
   - If solve-rate > 50% AND model accessed before 6 months ago → flag
     ContaminationSuspected.
   - (Conservative; ARC-AGI-2 explicitly resists training-data leakage but
     soft contamination per arxiv:2602.12413 possible).

5. **SSB action types** (re-use BenchmarkStarted + BenchmarkResult from D-19;
   no new SCHEMA bump).

6. **Decision doc** workflow/reasoning/D-20-decisions.md (3 decisions):
    - (a) Why cloud Sonnet only: local 7B insufficient for ARC; benchmark
      gates ELAI's mode-routing infrastructure, not the underlying model.
    - (b) Why honest-evaluation pass criterion (not solve-rate threshold):
      ARC trails frontier; meaningful solve rate threshold would gate on
      future model improvements. D-20 gates ON the framework (refusal SLA,
      no auto-discovery-publish, sycophancy hygiene), not solve rate.
    - (c) Why 50-task subset (not full 400): cost; subset still
      representative.

## WIRING_CONTRACT

produces_symbols:
  - ARCAGI2Runner + ARCReport + ARCVerdict
  - run + contamination heuristic
  - `elai bench arc-agi2` CLI
consumes_symbols:
  - CloudClient (B-37 cloud-mode)
  - GroundingMode dispatch (D-10)
  - NoveltyClass (D-15)
  - RefusalDecision (D-16)
  - SycophancyFlag (D-14)
  - SsbWriter
reachable_from:
  - `elai bench arc-agi2` CLI
  - CI workflow (optional; expensive)
stub_ban:
  - ARCAGI2Runner MUST NOT auto-publish DiscoveryCandidate (D-15 rule)
  - solve-rate report MUST cite human baseline + ELAI's framework caveats
  - 50-task subset MUST be a documented stratified sample
observable_effect:
  trigger: `elai bench arc-agi2`
  expect_ssb: 1 BenchmarkStarted; per-task ModeSelected + ClaimDispatched;
              aggregated BenchmarkResult with novelty/refusal/sycophancy
              distributions
  expect_artifact: .elai_cc/benchmarks/arc_agi2/reports/{date}.json with
                   honest report; no auto-publish of any DiscoveryCandidate

## DETERMINISTIC_CHECK

```bash
rg "struct ARCAGI2Runner|fn run" .elai_cc/crates/orchestrator/src/benchmarks/arc_agi2_runner.rs
ls .elai_cc/benchmarks/arc_agi2/fixtures/*.json | wc -l
# Expected: >= 50
test -f .elai_cc/crates/cli/src/bench_arc.rs
cargo check --workspace 2>&1 | tail -3
```

## ACCEPTANCE

- [ ] ARCAGI2Runner compiles; unit tests on synthetic ArcTask
- [ ] 50 ARC-AGI-2 fixtures present
- [ ] `elai bench arc-agi2` runs successfully via cloud Sonnet
- [ ] cargo check --workspace: zero new errors
- [ ] cargo test -p elai_orchestrator -- arc: all pass
- [ ] Cloud smoke test A: full 50-task ARC run with mode-routing on; report
      generated with solve-rate + refusal-rate + mode distribution + novelty
      distribution
- [ ] Cloud smoke test B: refusal_rate in healthy band [5%, 50%] (D-17)
- [ ] Cloud smoke test C: zero auto-publishes of DiscoveryCandidate
- [ ] Cloud smoke test D: sycophancy_flag_count = 0 (or near-zero;
      single-turn ARC tasks shouldn't trigger sycophancy)
- [ ] Decision doc workflow/reasoning/D-20-decisions.md

## Files touched

- ADD: .elai_cc/crates/orchestrator/src/benchmarks/arc_agi2_runner.rs
- ADD: .elai_cc/crates/cli/src/bench_arc.rs
- ADD: .elai_cc/benchmarks/arc_agi2/fixtures/{task_id}.json × 50
- MODIFY: .elai_cc/crates/cli/src/commands.rs (SlashCommand::BenchArcAgi2)
- ADD: workflow/reasoning/D-20-decisions.md

## Out-of-scope

- Full 400-task ARC run (DEFERRED; cost; 50-task subset representative)
- ARC-AGI-1 baseline (DEFERRED; AGI-2 is more current)
- Synthetic ARC-style generation (DEFERRED to research extension)

## Next

D-21 (mode-routing coverage benchmark).
