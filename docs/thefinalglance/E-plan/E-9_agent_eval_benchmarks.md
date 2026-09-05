# E-9: External agent-eval suite — SWE-bench-Live, LiveCodeBench v6, tau2-bench, BFCL v3, GAIA, ClawsBench harness/model split
Objective: ship a contamination-resistant external benchmark suite that anchors ELAI's internal D-19..D-25 measurements against 2026 public leaderboards, with ClawsBench-style harness-vs-model decomposition wired as the mandatory companion to every score; gates B-N promotion and any harness/scaffold change against measured deltas with paired-bootstrap p<0.05 significance testing.
Estimated effort: 3 sessions, ~12 h (split as bench harness + 3 priority suites; tau2 + BFCL + GAIA; ClawsBench split + DS-CP + reporting)
Pre-requisites: B-37 (Multi-Runtime Backend Abstraction, DONE — cloud route required for ceiling probes), B-23 (self-mod for promotion gating, PLANNED but optional), D-22 (conformal calibration validation, PLANNED — calibration baseline used by DS-CP). PerRoleRuntime + runtime orchestrator present per memory:project_b37_shipped.
Parallel-safe with: E-6, E-7 (their fixes are bench-targets), E-8 (its optimized templates are bench-targets), E-10.
Source report: workflow/improvement-sweep-2026-05-19/reports/09-agent-eval-benchmarks.md

## Context

The 2026 leaderboard landscape collapses to a few contamination-resistant signals: SWE-bench-Live (arXiv:2505.23419, monthly refresh; Verified is now considered contaminated and OpenAI stopped reporting), LiveCodeBench v6 (arXiv:2403.07974, time-stamped), tau2-bench (sierra-research/tau2-bench, retail 0.862 / telecom 0.993 Anthropic SOTA), BFCL v3 (gorilla.cs.berkeley.edu, GLM 4.5 SOTA 76.7, Qwen3-32B 75.7 — local-7B reachable), and GAIA Level 1+2 (arXiv:2311.12983, Claude Sonnet 4.5 leads at 74.6%). ClawsBench (arXiv:2604.05172) is the first benchmark to formally decompose scaffold vs model: top-5 models within 10pp on success (53-63%) but scaffold effect is +39-63pp — the harness dominates the score, so an eval that doesn't fix-then-vary-then-fix the harness is measuring the wrong thing. Soft contamination (arXiv:2602.12413) reports 78% of CodeForces and 50% of ZebraLogic have semantic duplicates that 9-gram filters miss. DS-CP (arXiv:2510.05566) provides reweighted-by-proximity conformal prediction under distribution shift. GSM8K-Platinum (gradientscience.org/gsm8k-platinum) cleaned 110/219 broken GSM8K items so the raw GSM8K is no longer a hard gate. Multi-day persistence and cost-aware quality are not yet well-measured by any public eval; DeepPlanning (arXiv:2601.18137) is the only 2026 eval attempting 2-7 day horizons. E-9 wires SWE-bench-Live + LiveCodeBench v6 + tau2-bench + BFCL v3 + GAIA L1/L2 + ClawsBench split + GSM8K-Platinum + MATH-500 + HLE no-tools ceiling probe + DeepPlanning multi-day as ELAI's external regression suite, plus DS-CP coverage check as the calibration validator, with each suite gated by paired-bootstrap significance.

## Existing state (B/C/D coverage)

- DELTA on D-19 (PRMBench adversarial gaming, PLANNED): E-9 picks up D-19's PRMBench fixtures as one of the 11 external suites in this E-N.
- DELTA on D-20 (ARC-AGI-2 OOD probe, PLANNED): E-9 demotes ARC-AGI-2 from a hard regression gate to a ceiling-drift tracker per report's recommendation (24% best SOTA on 7B-tier is too noisy as a gate).
- DELTA on D-21 (Mode-routing coverage benchmark, PLANNED): E-9 references D-21's hand-labeled mode set as ELAI's internal complement to BFCL v3 + tau2-bench external.
- DELTA on D-22 (Conformal calibration validation, PLANNED): E-9 replaces plain conformal with DS-CP (arXiv:2510.05566) reweighted-by-proximity per report's recommendation.
- DELTA on D-23 (Sycophancy detection benchmark, PLANNED): E-9 wires Sharma et al public probes as one of the 11 suites.
- DELTA on D-24 (Cross-domain canary regression, PLANNED): E-9 adds paraphrase decontamination (cosine 0.85 per E-7 task 6) as the canary's contam filter.
- DELTA on D-25 (Determinism + archive tamper suite, PLANNED): no change here; E-9 references it but does not modify.
- NEW external suite at .elai_cc/benches/external/ (parallel to .elai_cc/benches/model_capability already touched per git status).
- NEW ClawsBench-style harness-vs-model decomposition harness — flagged in the report as the mandatory companion.
- NEW paired-bootstrap p<0.05 gating with n-power calibration per suite.

## Reads (mandatory before coding)

- F:\.ELAI_workflow\workflow\improvement-sweep-2026-05-19\reports\09-agent-eval-benchmarks.md
- .elai_cc/crates/cli/src/run_subcommand.rs (existing run pipeline used for SWE-bench-Live integration)
- .elai_cc/crates/orchestrator/src/per_role_runtime.rs (cloud vs local routing for ceiling probes)
- .elai_cc/benches/model_capability/run_benchmark.py (existing bench harness pattern; reference for new external benches)
- .elai_cc/crates/orchestrator/src/symbol_graph.rs (target of SWE-bench-Live regression for B-N changes touching symbol graph)
- F:\.ELAI_workflow\docs\thefinalglance\D-plan\D-SUMMARY_INDEX.md (D-19..D-25)
- arXiv:2505.23419 SWE-bench-Live
- arXiv:2403.07974 LiveCodeBench
- arXiv:2311.12983 GAIA
- arXiv:2604.05172 ClawsBench (focused.io/lab/agent-benchmarks-measure-the-harness)
- arXiv:2602.12413 soft contamination
- arXiv:2510.05566 DS-CP
- arXiv:2501.03124 PRMBench (D-19 grounding)
- arXiv:2310.13548 Sharma sycophancy probes
- arXiv:2601.18137 DeepPlanning (multi-day)
- gradientscience.org/gsm8k-platinum (110/219 cleanup)
- sierra-research/tau2-bench repo
- gorilla.cs.berkeley.edu BFCL leaderboard
- benchlm.ai/benchmarks/osWorldVerified (optional desktop probe)

## Tasks

1. Define BenchmarkSuite trait + registry at .elai_cc/crates/orchestrator/src/external_benches.rs (~220 LOC). Trait: `fn name(&self) -> &str; fn version(&self) -> &str; fn run(&self, model: &BenchTarget, harness: &HarnessConfig) -> BenchmarkResult; fn paired_bootstrap_significance(&self, a: &BenchmarkResult, b: &BenchmarkResult) -> SignificanceVerdict`. Registry holds 11 suite implementations.

2. SWE-bench-Live runner at .elai_cc/benches/external/swe_bench_live.py (~280 LOC). Pulls last 50 fresh issues from https://swe-bench-live.github.io/ Multi-Language leaderboard (HTTP cache TTL = 24h). Verifies content-hash + temporal cutoff per E-7's contam filter. Runs against existing CLI run_subcommand pipeline. Threshold: must not regress more than 2 points vs prior month (paired bootstrap p < 0.05). Emits SsbActionType::SweBenchLiveRunCompleted.

3. LiveCodeBench v6 runner at .elai_cc/benches/external/livecodebench_v6.py (~240 LOC). Last-60-days slice; hooks PerRoleRuntime executor. Sub-tasks: code-gen, self-repair, test-output-prediction. Local-7B floor 30%, cloud route floor 45%. Emits SsbActionType::LiveCodeBenchRunCompleted.

4. tau2-bench runner at .elai_cc/benches/external/tau2_bench.py (~260 LOC). Telecom + retail subsets. Wires into orchestrator policy-check pass. Rubric grades policy adherence, NOT outcome — booking a flight while violating change-fee policy fails. Thresholds: retail >= 0.70 cloud / 0.55 local; telecom >= 0.85 cloud / 0.55 local. Anthropic SOTA reference: 0.862 retail / 0.993 telecom. Emits SsbActionType::Tau2BenchRunCompleted.

5. BFCL v3 runner at .elai_cc/benches/external/bfcl_v3.py (~200 LOC). Multi-turn slice; per-role function-calling correctness; complements D-21 internal mode-routing. Threshold: >= 70% (GLM 4.5 SOTA 76.7, Qwen3-32B 75.7). Emits SsbActionType::BfclV3RunCompleted.

6. GAIA L1+L2 runner at .elai_cc/benches/external/gaia_l1_l2.py (~220 LOC). HAL framework (https://hal.cs.princeton.edu/gaia). Contract-test fixture under crates/contracts/. Thresholds: L1 >= 50%, L2 >= 30%. SOTA reference Sonnet 4.5 74.6% combined. Emits SsbActionType::GaiaRunCompleted.

7. ClawsBench harness-vs-model decomposition at .elai_cc/benches/external/clawsbench_split.py (~320 LOC). Run cross product {model A, model B} x {harness X, harness Y}. Decompose variance via 2-way ANOVA into model-effect + harness-effect + interaction. Reports if scaffold dominates (per published +39-63pp scaffold effect). Mandatory companion to every other suite when a harness change is in the diff. Emits SsbActionType::ClawsBenchSplitCompleted with model_effect_pp + harness_effect_pp.

8. PRMBench adversarial fixture at .elai_cc/benches/external/prmbench_adversarial.py (~180 LOC; merges with D-19). Pulls Deception Resistance + Circular Logic subsets. Gates D-2 PPM promotion at >= published baseline. Emits SsbActionType::PrmBenchRunCompleted.

9. Sharma sycophancy probes at .elai_cc/benches/external/sycophancy_probes.py (~160 LOC; merges with D-23). 4 behaviors: feedback flip, mimicry, answer-flip, false-claim agreement. Threshold: flip rate < 15% on TruthfulQA-derived feedback probes. Emits SsbActionType::SycophancyProbeCompleted.

10. GSM8K-Platinum + MATH-500 at .elai_cc/benches/external/math_floors.py (~160 LOC). Reasoning floor probe; uses cleaned GSM8K-Platinum (110/219 broken items corrected) not raw GSM8K. Threshold: GSM8K-Platinum >= 90%, MATH-500 >= 50% locally. Emits SsbActionType::MathFloorsCompleted.

11. HLE no-tools ceiling probe at .elai_cc/benches/external/hle_ceiling.py (~140 LOC). 50-question slice, quarterly only — NOT a regression gate. Tracks ceiling drift relative to Claude Opus 4.7 46.9% no-tools SOTA. Emits SsbActionType::HleCeilingTracked.

12. DeepPlanning multi-day at .elai_cc/benches/external/deep_planning.py (~200 LOC). Long-horizon contract test for B-N plans spanning phases. Constraint-satisfaction threshold >= 70% on 2-3 day plans. Emits SsbActionType::DeepPlanningCompleted.

13. DS-CP coverage validator at crates/orchestrator/src/ds_cp_validator.rs (~240 LOC; merges with D-22). Replaces plain conformal with reweighted-by-proximity per arXiv:2510.05566. 3 held-out distributions (in-domain, near-OOD, far-OOD) per D-22. Validates coverage gap < 5pp under benchmarked shifts. Emits SsbActionType::DsCpCoverageMeasured.

14. Paired-bootstrap significance harness at crates/orchestrator/src/bench_significance.rs (~180 LOC). For each suite, calculate per-task n required for p<0.05 on a 2-point delta given observed variance; emit SsbActionType::BenchPowerCalibrated with required_n + observed_n. Refuses to call a result a "regression" if n < required_n; instead emits LowPowerNoConclusion.

15. CLI: `elai bench external [--suite NAME] [--harness-vary] [--p-target 0.05]` at crates/cli/src/bench_cmd.rs (~220 LOC delta). Spawns Python runners; tails SSB; reports per-suite verdicts; final pass/fail/low-power table.

16. Dashboard ExternalBenchPanel.svelte (~260 LOC) showing the 11 suites in a 4x3 grid (one cell empty), per-suite latest score + SOTA reference + paired-bootstrap p; click-through to the run history. Tauri command get_external_bench_history(suite_name) returns Vec<BenchmarkResult>.

17. Anti-LLM-as-judge-bias rotation: when an LLM judges a result (HLE, sycophancy probes), the harness MUST rotate option positions and ID tokens per call (cite arXiv:2410.15393, arXiv:2504.03846). Test contract enforces rotation.

## WIRING_CONTRACT

produces_symbols:
  - orchestrator::external_benches::BenchmarkSuite (trait)
  - orchestrator::external_benches::HarnessConfig
  - orchestrator::external_benches::BenchmarkResult
  - orchestrator::bench_significance::paired_bootstrap_p(a, b, n_iter) -> f64
  - orchestrator::ds_cp_validator::ds_cp_coverage(calib_set, shift_set, alpha) -> CoverageReport
  - SsbActionType::SweBenchLiveRunCompleted
  - SsbActionType::LiveCodeBenchRunCompleted
  - SsbActionType::Tau2BenchRunCompleted
  - SsbActionType::BfclV3RunCompleted
  - SsbActionType::GaiaRunCompleted
  - SsbActionType::ClawsBenchSplitCompleted
  - SsbActionType::PrmBenchRunCompleted
  - SsbActionType::SycophancyProbeCompleted
  - SsbActionType::MathFloorsCompleted
  - SsbActionType::HleCeilingTracked
  - SsbActionType::DeepPlanningCompleted
  - SsbActionType::DsCpCoverageMeasured
  - SsbActionType::BenchPowerCalibrated
consumes_symbols:
  - crates/cli/src/run_subcommand.rs (SWE-bench-Live calls existing run pipeline)
  - crates/orchestrator/src/per_role_runtime.rs (cloud vs local routing)
  - crates/orchestrator/src/promotion_gate.rs (E-7) — B-N promotion gated on suite verdicts
  - dashboard-v2/src-tauri::get_external_bench_history
reachable_from:
  - cli::main::main `elai bench external ...`
  - any B-N or E-N promotion-gate path (calls suite registry on a model/harness change)
stub_ban:
  - "Verified-only" SWE-bench score reporting (per report's anti-pattern)
  - "n-gram" alone contam check absent (must include paraphrase + canary per E-7 task 6)
  - any "GSM8K" reference outside the GSM8K-Platinum cleaned set
  - any LLM-as-judge call without explicit position+ID rotation
frontend_pair: crates/dashboard-v2/src/lib/components/ExternalBenchPanel.svelte
observable_effect:
  trigger: "elai bench external --suite swe-bench-live --harness-vary"
  expect_ssb: "SsbActionType::SweBenchLiveRunCompleted and SsbActionType::ClawsBenchSplitCompleted within 30 min; paired-bootstrap p reported; BenchPowerCalibrated also fires"
  expect_artifact: "stdout shows per-suite verdict with p-value; dashboard ExternalBenchPanel renders 11-suite grid with latest scores + SOTA references"

## DETERMINISTIC_CHECK

```bash
# 1. 11 suites registered
cargo run -p elai-cli -- bench external --list --json | jq '.suites | length'
# Expected: 11

# 2. Paired-bootstrap p reported per suite
cargo run -p elai-cli -- bench external --suite gsm8k_platinum --json | jq '.p_value'
# Expected: a float between 0 and 1.

# 3. ClawsBench split runs cross-product
cargo run -p elai-cli -- bench external --suite clawsbench_split --harness-vary --json | jq '.model_effect_pp, .harness_effect_pp'
# Expected: two non-null floats.

# 4. SWE-bench-Live time-stamped + temporal cutoff applied
rg "released_at|temporal_cutoff" .elai_cc/benches/external/swe_bench_live.py
# Expected: at least one match.

# 5. No raw GSM8K reference outside platinum cleanup
rg "gsm8k" .elai_cc/benches/external --type py -i
rg "platinum|cleaned" .elai_cc/benches/external/math_floors.py
# Expected: every gsm8k reference paired with platinum/cleaned context.

# 6. LLM-as-judge rotation contract
cargo test -p elai-orchestrator external_benches::tests::judge_rotates_positions_and_ids
# Expected: PASS.

# 7. DS-CP coverage gap < 5pp on calibration fixture
cargo run -p elai-cli -- bench external --suite ds_cp --json | jq '.coverage_gap_pp'
# Expected: < 5.

# 8. Power-calibration refuses regression call when n < required_n
cargo test -p elai-orchestrator bench_significance::tests::low_power_no_conclusion
# Expected: PASS.

# 9. Verified-only SWE-bench not reported
rg "swe.bench.verified" .elai_cc/benches/external --type py -i
# Expected: zero hits, OR only inside a comment explaining why we don't use it.
```

## ACCEPTANCE

- [ ] 11 external benchmark suites registered; `elai bench external --list` enumerates all 11.
- [ ] SWE-bench-Live regression gate refuses promotion if month-over-month delta < -2 points AND paired-bootstrap p < 0.05.
- [ ] LiveCodeBench v6 floor (local 30%, cloud 45%) gates the executor; per-language breakdown surfaced.
- [ ] tau2-bench grades policy adherence not outcome; retail/telecom both reported with separate thresholds.
- [ ] BFCL v3 multi-turn slice gates per-role function-calling; floor 70%.
- [ ] GAIA L1+L2 contract test fixture; L1 >=50% AND L2 >=30% required for "shipped" tag promotion.
- [ ] ClawsBench harness-vs-model split is the mandatory companion to any harness/scaffold change; emits model_effect_pp + harness_effect_pp; refuses to attribute a score delta to the model if model spread < 10pp within harness.
- [ ] PRMBench Deception Resistance + Circular Logic subsets feed D-2 PPM promotion gate.
- [ ] Sharma sycophancy probes wired; flip rate < 15% gates auditor + planner persona changes.
- [ ] GSM8K-Platinum + MATH-500 only — no raw GSM8K anywhere.
- [ ] HLE no-tools 50-q slice runs quarterly via scheduled task (NOT a regression gate; trend only).
- [ ] DeepPlanning constraint-satisfaction >= 70% on 2-3 day plan fixtures.
- [ ] DS-CP replaces plain conformal in D-22 path; coverage gap < 5pp under shifts.
- [ ] Paired-bootstrap power-calibration; LowPowerNoConclusion emitted when n insufficient; never silently call a regression.
- [ ] LLM-as-judge calls rotate option positions + ID tokens (anti-bias rule); contract test asserts.
- [ ] Anti-slopsquatting verified for every new Python dep (gepa already from E-8 if shared; new: bench-specific libs verified via pypi.org/pypi/{pkg}/json).

## Tests to add

- crates/contracts/tests/external_bench_registry_contract.rs: 11 suites registered, version strings present
- crates/orchestrator/tests/bench_significance_low_power.rs: n < required_n triggers LowPowerNoConclusion
- crates/orchestrator/tests/clawsbench_split_anova.rs: 2-way ANOVA decomposition matches a known synthetic dataset
- crates/orchestrator/tests/ds_cp_coverage_under_shift.rs: coverage gap < 5pp on a 3-distribution synthetic
- crates/orchestrator/tests/judge_rotation_contract.rs: every LLM-as-judge call rotates positions
- crates/orchestrator/tests/swe_bench_live_temporal_cutoff.rs: contaminated task (older than cutoff) rejected
- python/benches/external/tests/test_tau2_policy_grading.py: violating change-fee policy fails the rubric even with successful booking
- python/benches/external/tests/test_sycophancy_probes.py: 4 behaviors detected on a labeled fixture
- python/benches/external/tests/test_gsm8k_platinum_only.py: GSM8K-Platinum 110-corrected set used; raw GSM8K refused
- crates/dashboard-v2/src/lib/components/ExternalBenchPanel.test.ts: 11-suite grid renders + click-through to history

## Files touched

- CREATE: crates/orchestrator/src/external_benches.rs (~220 LOC)
- CREATE: crates/orchestrator/src/bench_significance.rs (~180 LOC)
- CREATE: crates/orchestrator/src/ds_cp_validator.rs (~240 LOC)
- CREATE: crates/cli/src/bench_cmd.rs (~220 LOC delta on existing if present)
- CREATE: crates/dashboard-v2/src/lib/components/ExternalBenchPanel.svelte (~260 LOC)
- CREATE: .elai_cc/benches/external/swe_bench_live.py (~280 LOC)
- CREATE: .elai_cc/benches/external/livecodebench_v6.py (~240 LOC)
- CREATE: .elai_cc/benches/external/tau2_bench.py (~260 LOC)
- CREATE: .elai_cc/benches/external/bfcl_v3.py (~200 LOC)
- CREATE: .elai_cc/benches/external/gaia_l1_l2.py (~220 LOC)
- CREATE: .elai_cc/benches/external/clawsbench_split.py (~320 LOC)
- CREATE: .elai_cc/benches/external/prmbench_adversarial.py (~180 LOC)
- CREATE: .elai_cc/benches/external/sycophancy_probes.py (~160 LOC)
- CREATE: .elai_cc/benches/external/math_floors.py (~160 LOC)
- CREATE: .elai_cc/benches/external/hle_ceiling.py (~140 LOC)
- CREATE: .elai_cc/benches/external/deep_planning.py (~200 LOC)
- CREATE: .elai_cc/benches/external/pyproject.toml (suite deps; all version-verified via pypi.org/pypi)
- MODIFY: crates/orchestrator/src/ssb.rs (+13 SsbActionType variants, ~130 LOC delta)
- MODIFY: crates/cli/src/main.rs (~25 LOC to register bench external subcommand)
- MODIFY: crates/dashboard-v2/src-tauri/src/lib.rs (~50 LOC for get_external_bench_history command)

## Citations / Grounding

- arXiv:2505.23419 (SWE-bench-Live) — monthly-refreshed Multi-Language leaderboard; grounds suite #2
- arXiv:2403.07974 (LiveCodeBench) — time-stamped contamination-free; mean 50.7, sd 23.5; grounds suite #3
- arXiv:2311.12983 (GAIA) — Claude Sonnet 4.5 leads 74.6%; grounds suite #5 thresholds
- sierra-research/tau2-bench — policy-adherence rubric; Anthropic SOTA 0.862 retail / 0.993 telecom; grounds suite #4
- gorilla.cs.berkeley.edu (BFCL v3) — GLM 4.5 SOTA 76.7, Qwen3-32B 75.7; grounds suite #5 threshold
- arXiv:2604.05172 (ClawsBench) — scaffold effect +39-63pp vs model effect 10pp; grounds mandatory harness split
- arXiv:2602.12413 (soft contamination) — 78% CodeForces + 50% ZebraLogic semantic duplicates; grounds paraphrase decontamination
- arXiv:2510.05566 (DS-CP) — reweighted-by-proximity conformal under shift; grounds D-22 replacement
- arXiv:2501.03124 (PRMBench) — 6216 problems, 83456 step-labels, 9 sub-dimensions; grounds D-19 fixture
- arXiv:2310.13548 (Sharma sycophancy) — 4 behaviors; grounds suite #9
- arXiv:2601.18137 (DeepPlanning) — 2-7+ day horizon; grounds suite #12
- gradientscience.org/gsm8k-platinum — 110/219 broken items corrected; grounds GSM8K-Platinum-only rule
- arXiv:2410.15393 + arXiv:2504.03846 — LLM-as-judge position + ID bias; grounds rotation rule
- focused.io/lab/agent-benchmarks-measure-the-harness — "scores are measuring the harness, not the model"; grounds ClawsBench companion mandate

## Open questions / risks

- "What's the smallest SWE-bench-Live slice (n) that yields paired-bootstrap p<0.05 on a 2-point delta?" — report flagged this as open. E-9 emits BenchPowerCalibrated with required_n so the answer surfaces empirically; default n=50 may be too small for a 2-point detection.
- HLE no-tools 50-q slice is sampled from the full ~3000-q HLE; sampling variance could be high. Mitigation: use stratified sampling by Scale leaderboard category; treat as ceiling-drift tracker only, not a regression gate (per report).
- ARC-AGI-2 demoted from gate to tracker. Risk: ELAI's D-20 originally framed ARC-AGI-2 as a gate; E-9 demotes per report's "frontier-only benchmarks; 7B models score in single-digit %" guidance. If the user wants a hard ARC gate, must opt in via env var.
- tau2-bench policy-adherence rubric may not transfer to ELAI's internal SSB-action policy. Report flagged this as open question. E-9 ships the suite but logs per-policy-rule pass/fail so a custom rubric layer can be built later.
- DS-CP requires calibration corpus; D-30 (calibration bootstrap) is DEFERRED in D-SUMMARY. E-9 lands DS-CP code with synthetic calibration data; production deployment blocks on D-30.
- Cost-aware quality and multi-day-persistence are flagged as "NOT well-measured" by any public eval. DeepPlanning is the only attempt and still emulates a multi-day plan in one rollout. E-9 ships DeepPlanning but with a note that this is the best available, not a definitive multi-day measurement.
- gepa-style external dep verification (every Python suite) — anti-slopsquatting rule applies per dep; pyproject.toml comments record version + last-published date per dep at install time.
- The 11 suites running serially on a real workload is hours-to-days of wall-clock; CI must shard. E-9 ships the harness; CI sharding is a follow-on.
- "Ungrounded:" report did not flag DS-CP as Ungrounded but cautioned coverage gap > 5pp on high-dim LLM inputs per arXiv:2604.16217 (Beyond Surface Statistics) — E-9 surfaces this via dashboard so a user can see if DS-CP is failing in practice.

## Next

- E-9 unblocks any B-N or E-N promotion that touches model/harness — gates against the 11-suite verdict + paired-bootstrap p
- E-7 + E-8 outputs feed E-9 as bench targets — Darwinian-substrate candidates + GEPA-optimized templates each get their full suite run before promotion
- E-10 (critic) recommends collapsing 7-layer knowledge pipeline to 3 — E-9's ClawsBench harness-vs-model split is the empirical lever to decide if the simplification gains scaffold-effect points
- Future E-N: cost-aware quality eval (no public benchmark exists; ELAI may need to author its own $/task vs pass-rate Pareto)
- Future E-N: longitudinal user-trust drift over weeks (also no public benchmark; this is multi-day++ scope)
