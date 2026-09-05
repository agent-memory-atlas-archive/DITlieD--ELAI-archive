# D-5: BAI promotion — Beta-posterior Thompson sampling for template archive

Objective: replace MSI sigmoid Boltzmann (Phase 4H spec heuristic) with
Beta-posterior Thompson sampling + Successive Rejects + LUCB for promote/
retire decisions. Published regret bounds from Audibert COLT 2010 (T1) +
Variance-Dependent BAI (arxiv:2106.10417, T1). Strictly stronger than DGM's
sigmoid-Boltzmann selection (companion §3.7 lines 296-317).
Estimated effort: 2 sessions.
Pre-requisites: D-1 (ArchiveChain entries to track), D-3 (VerifierConsensusPass
  count as pass-rate input). spec-grounder verifies both DONE.
Parallel-safe with: D-6 (held-out store; BAI consumes held-out pass-rate but
  does not write held-out tasks), D-7 (cross-domain canary; same). NOT parallel-
  safe with D-4 (Tripwire stack auto-reverts the same archive that BAI promotes
  from — coordination via ArchiveChain locks).

## Context

Companion §3.7 (lines 296-317): MSI sigmoid Boltzmann is heuristic; BAI is
theory. Audibert et al COLT 2010 (T1) gives Successive Rejects + LUCB
regret bounds; Variance-Dependent BAI (arxiv:2106.10417, T1) extends with
per-arm variance awareness. Neither DGM, STaR, ReST^EM, nor ELAI current
spec uses Beta-posterior Thompson explicitly; adopting it is strictly
stronger on the selection axis.

The structural shape:
- Every Candidate ReasoningTemplate (D-1) has an ArmPosterior tracking
  pass_count + fail_count.
- Beta(pass + 1, fail + 1) is the posterior over the template's true
  pass-rate.
- Thompson sampling: sample θ_i ~ Beta(pass_i+1, fail_i+1) for each arm i;
  promote arm with highest sample to next stage (10 → 50 → 200 evals per
  companion §0).
- Promotion rule: Candidate → Promoted when
  credible_interval(arm.posterior, α=0.05).lower_bound > current_promoted_median
  + δ, where δ tuned per task family (typically 0.02-0.05 abs pass-rate).
- Successive Rejects: at each round, eliminate the worst-performing arm.
- LUCB: pull the arm with highest upper-confidence-bound vs the empirically
  best.

Partial foundation present and usable:
- D-1 ArchiveChain + ArchiveEntry.staged_eval_result has stage_10_pass/total,
  stage_50_pass/total, stage_200_pass/total placeholder fields.
- D-3 VerifierConsensusPass count = empirical pass signal.
- RetirementReason enum (D-1) already has Superseded variant for BAI-evicted
  arms.

## Reads

- F:\.ELAI_workflow\docs\specs\guide\reasoning-improvement-grounded-2026-05-19.md
  (§3.7 BAI; §1.14 Audibert + variance-dependent — lines 126-129 + 296-317)
- F:\.ELAI_workflow\workflow\papers\rstar-math-2501.04519-interface.md (D-2)
- F:\.ELAI_workflow\workflow\papers\statistical-limits-2510.04399-interface.md (D-1)
- .elai_cc/crates/orchestrator/src/archive_chain.rs (D-1 ArchiveEntry +
  StagedEvalResult; BAI populates the stage_*_pass/total fields)
- .elai_cc/crates/orchestrator/src/reasoning_template.rs (D-1 archive_status
  transition logic)
- .elai_cc/crates/orchestrator/src/verification_result.rs (D-3 VerifierConsensusPass
  count source)

## Paper-ingestor pre-step

Spawn paper-ingestor with mode=paper for:
- Audibert et al COLT 2010 BAI (PDF URL: https://www.learningtheory.org/colt2010/papers/59Audibert.pdf;
  paper-ingestor handles non-arxiv PDFs)
- arxiv:2106.10417 Variance-Dependent BAI

Verify workflow/papers/{audibert-colt-2010-bai,variance-bai-2106.10417}-interface.md.

## Tasks

1. **ArmPosterior types** in new file
   .elai_cc/crates/orchestrator/src/bai.rs:
   - `struct ArmPosterior { template_id: TemplateId, pass_count: u32,
     fail_count: u32, last_updated: DateTime<Utc> }`
   - `impl ArmPosterior`: alpha() = pass_count + 1; beta() = fail_count + 1;
     mean() = alpha() / (alpha() + beta()) as f64;
     credible_interval(alpha_level: f32) -> (f32, f32) — use
     statrs::distribution::Beta if available, else compute via
     incomplete-beta-function (hand-rolled, <40 LOC).
   - Verify statrs in Cargo.toml; if absent, ask user before adding (per
     CLAUDE.md no-new-deps-without-confirmation).
   - `impl ArmPosterior`: update(passed: bool) -> Self; immutable update
     pattern.

2. **Thompson sampling** in bai.rs:
   - `fn sample_beta(rng: &mut SeededRng, alpha: f32, beta: f32) -> f32`:
     sample from Beta(alpha, beta) via gamma-ratio method (G_a/(G_a+G_b)
     where G ~ Gamma); use SeededRng from C-15 for determinism in tests.
   - `fn thompson_select(arms: &[ArmPosterior], rng: &mut SeededRng) ->
     Option<TemplateId>`: sample θ_i for each arm; return arm with max θ;
     None on empty input.

3. **Successive Rejects** in bai.rs:
   - `fn successive_rejects(arms: Vec<ArmPosterior>, eval_budget: u32) ->
     Vec<TemplateId>`: per Audibert COLT 2010 §3; round-by-round elimination;
     at round k, eliminate worst arm by empirical mean; remaining arms get
     n_k pulls each (n_k = budget / (K-k)); return ordered Vec of survivors.

4. **LUCB1** in bai.rs:
   - `fn lucb1_next_pull(arms: &[ArmPosterior], delta: f32) -> Option<TemplateId>`:
     find empirically-best arm by mean; find competitor with highest UCB =
     mean_i + sqrt(log(1/delta) / (2*n_i)); return competitor (the arm to
     pull next, balancing exploration vs exploitation).

5. **BaiPromotionEngine** in bai.rs:
   - `struct BaiPromotionEngine { arms: HashMap<TemplateId, ArmPosterior>,
     promoted: Vec<TemplateId>, retired: Vec<TemplateId>, eval_budget: u32,
     delta_per_family: HashMap<Domain, f32>, rng: SeededRng }`
   - `fn ingest_eval_result(template_id, passed: bool)`: update arm posterior.
   - `fn promote_round(stage: PromotionStage) -> Vec<TemplateId>`: per
     companion §3.7 promotion rule; emit BaiPromotionDecision SSB row per
     arm; transition status Candidate → Promoted on rule satisfaction;
     ArchiveChain.append the promotion event.
   - `fn retire_round(stage: PromotionStage) -> Vec<TemplateId>`: invoke
     successive_rejects against bottom quartile; retire with
     RetirementReason::Superseded; ArchiveChain.retire.
   - `enum PromotionStage { Stage10, Stage50, Stage200 }`

6. **SSB action types** in ssb.rs (SCHEMA_VERSION 50→51):
   - `ArmPosteriorUpdated { template_id, pass_count: u32, fail_count: u32,
     mean: f64, ci_lower: f64, ci_upper: f64 }`
   - `ThompsonSampled { round_id: Uuid, sampled_arms: Vec<(TemplateId, f64)>,
     selected: TemplateId }`
   - `BaiPromotionDecision { template_id, decision: String, stage: String,
     evidence_pass: u32, evidence_total: u32, audit_id: Uuid }`
   - Round-trip tests.

7. **elai_cc.toml [bai] section**:
   - `[bai]\neval_budget = 200\nci_alpha = 0.05\n# per-family delta:\n
     delta_code = 0.05\ndelta_math = 0.03\ndelta_prose = 0.05\n
     delta_mixed = 0.04`
   - L3-LOCKED.

8. **Determinism**: BAI engine MUST use SeededRng (from C-15 + D-1) for
   all sampling. Two runs with same ELAI_SEED produce byte-identical
   promotion decisions. Test: 20-arm synthetic stream; ELAI_SEED=42 twice;
   diff promotion sequences → empty.

9. **Unit tests**:
   - ArmPosterior.mean for (0,0) = 0.5 (Beta(1,1) prior); for (10, 0) ≈ 0.917;
     for (0, 10) ≈ 0.083.
   - credible_interval(0.05) on (50, 50) is symmetric around 0.5; on (95, 5)
     skewed high.
   - sample_beta deterministic given fixed seed.
   - thompson_select returns deterministic result given fixed seed.
   - successive_rejects on 4 arms with budget=12 produces correct round
     allocation (4, 3, 2, 1 surviving arms per round per Audibert §3 formula).
   - lucb1_next_pull returns competitor (not empirically-best) when arms have
     similar means but different confidence widths.
   - BaiPromotionEngine: 5-arm 100-step synthetic stream where one arm has
     true rate 0.9 + four arms 0.5; assert that arm wins promotion within
     budget; assert SeededRng makes result deterministic.

10. **Integration test** in
    .elai_cc/crates/orchestrator/tests/bai_synthetic_regret.rs:
    - 10-arm bandit benchmark; arms with rates [0.9, 0.85, 0.7, 0.65, 0.5,
      0.5, 0.4, 0.3, 0.2, 0.1]. Budget = 500 pulls. 100 trials with different
      seeds.
    - Assert: BAI cumulative regret ≤ published Audibert bound
      O(sqrt(K * budget * log(budget))) by factor of 2× (conservative).
    - Assert: best arm selected as Promoted in ≥85% of trials.

11. **Decision doc** workflow/reasoning/D-5-decisions.md (3 decisions):
    - (a) Why Beta-Bernoulli posterior (not Gaussian on continuous score):
      VerifierConsensusPass is binary (Pass/Reject); Beta is conjugate.
    - (b) Why per-family delta (not global): different domains have different
      noise floors per companion §3.7 + UC-2; tuning per Domain matches.
    - (c) Why retain MSI as a fallback signal (NOT as the gate): MSI sigmoid
      Boltzmann is heuristic; downgrade to a "secondary signal" for operator
      visibility; BAI is the primary gate.

## WIRING_CONTRACT

produces_symbols:
  - ArmPosterior + impl (alpha, beta, mean, credible_interval, update)
  - sample_beta + thompson_select + successive_rejects + lucb1_next_pull
  - BaiPromotionEngine + PromotionStage
  - SsbActionType::ArmPosteriorUpdated + ::ThompsonSampled + ::BaiPromotionDecision
    + payloads
  - SCHEMA_VERSION bump 50→51
  - [bai] elai_cc.toml section
consumes_symbols:
  - ArchiveChain (D-1) for promote/retire append
  - ReasoningTemplate.archive_status (D-1)
  - VerifierConsensusPass count (D-3) as pass signal
  - SeededRng (C-15 + D-1) for determinism
  - StagedEvalResult.stage_*_pass/total fields (D-1)
reachable_from:
  - task_runner.rs after-eval hook (every VerifierConsensusPass/Reject)
  - elai_cc.toml [bai] section
  - SeededRng with ELAI_SEED env var
stub_ban:
  - sample_beta MUST use SeededRng, never thread_rng() (defends determinism)
  - BaiPromotionEngine MUST NOT skip promote_round on empty arms (gracefully
    return empty Vec, not panic)
  - credible_interval MUST handle small-sample case (pass=0, fail=0 returns
    (0, 1) per Beta(1,1) uniform; not NaN)
  - delta_per_family MUST be loaded from elai_cc.toml; agent cannot edit
observable_effect:
  trigger: 10-arm synthetic regret benchmark with seeded RNG
  expect_ssb: ≥100 ArmPosteriorUpdated rows; ≥1 BaiPromotionDecision row per
              survived arm; deterministic across runs with same seed
              (two runs → identical SSB diff)
  expect_artifact: bai integration test asserts cumulative regret ≤ Audibert
                   bound × 2 (conservative)

## DETERMINISTIC_CHECK

```bash
# 1. SCHEMA_VERSION 51
rg "SCHEMA_VERSION: i64 = 51" .elai_cc/crates/orchestrator/src/ssb.rs

# 2. 3 new SSB variants
rg "ArmPosteriorUpdated|ThompsonSampled|BaiPromotionDecision" \
  .elai_cc/crates/orchestrator/src/ssb.rs | wc -l
# Expected: >= 12

# 3. BAI types + algorithms
rg "struct ArmPosterior|fn sample_beta|fn thompson_select|fn successive_rejects|fn lucb1_next_pull" \
  .elai_cc/crates/orchestrator/src/bai.rs

# 4. SeededRng use (no thread_rng)
rg "SeededRng" .elai_cc/crates/orchestrator/src/bai.rs
rg "thread_rng\|rand::random" .elai_cc/crates/orchestrator/src/bai.rs
# Expected: SeededRng >= 1, thread_rng = 0

# 5. elai_cc.toml [bai] section
rg "\[bai\]" .elai_cc/elai_cc.toml

# 6. Build + scoped tests
cargo check --workspace 2>&1 | tail -3
cargo test -p elai_orchestrator -- bai 2>&1 | grep "test result"
cargo test -p elai_cc_contracts -- bai_promotion 2>&1 | grep "test result"

# 7. Paper-ingestor outputs
test -f workflow/papers/audibert-colt-2010-bai-interface.md && \
  test -f workflow/papers/variance-bai-2106.10417-interface.md && echo OK
```

## ACCEPTANCE

- [ ] SCHEMA_VERSION_PUBLIC == 51
- [ ] ArmPosterior + Beta posterior math correct (validated against hand-
      computed expectations)
- [ ] sample_beta + thompson_select deterministic under fixed SeededRng
- [ ] successive_rejects round allocation matches Audibert §3 formula
- [ ] lucb1_next_pull selects competitor over empirical best when confidence
      widths differ
- [ ] BaiPromotionEngine: 5-arm with one true 0.9 arm → Promoted within budget;
      4 inferior arms → Superseded
- [ ] Integration test (10-arm 500-pull regret benchmark, 100 trials):
      cumulative regret ≤ Audibert bound × 2; best arm selected ≥85% trials
- [ ] All 3 SSB variants round-trip
- [ ] elai_cc.toml [bai] section loaded; defaults applied
- [ ] cargo check --workspace: zero new errors
- [ ] cargo test -p elai_orchestrator -- bai: all pass, ≥15 new tests
- [ ] Cloud smoke test A: 50-cycle bootstrap with synthetic templates; verify
      ArmPosteriorUpdated rows; verify ≥1 BaiPromotionDecision row; verify
      ArchiveChain shows correct status transitions
- [ ] Cloud smoke test B: two cloud /task runs with ELAI_SEED=42 + identical
      synthetic stream; diff SSB DBs (only timestamps differ); BAI rows
      byte-identical
- [ ] Paper-ingestor outputs (audibert, variance-bai) present
- [ ] Decision doc workflow/reasoning/D-5-decisions.md (3 decisions)

## Files touched

- ADD: .elai_cc/crates/orchestrator/src/bai.rs
- MODIFY: .elai_cc/crates/orchestrator/src/ssb.rs (SCHEMA_VERSION 50→51;
    3 new variants + payloads)
- MODIFY: .elai_cc/crates/orchestrator/src/task_runner.rs (after-eval hook
    feeds BaiPromotionEngine)
- MODIFY: .elai_cc/elai_cc.toml ([bai] section)
- MODIFY: .elai_cc/crates/orchestrator/src/lib.rs (pub mod bai)
- ADD: .elai_cc/crates/orchestrator/tests/bai_synthetic_regret.rs
- ADD: .elai_cc/crates/contracts/tests/bai_promotion_contract.rs
- ADD: workflow/reasoning/D-5-decisions.md
- ADD: workflow/papers/{audibert-colt-2010-bai,variance-bai-2106.10417}-interface.md
- Possibly: add statrs to Cargo.toml IF user confirms (no in-tree Beta dist
    available; otherwise hand-rolled incomplete-beta)

## Out-of-scope

- D-29 Hierarchical BAI (non-IID rewards; research-grade; DEFERRED)
- D-28 Adversarial example farming (deferred from companion §3)
- Reward-model retraining (not in companion D-plan scope)

## Next

D-6 (held-out task store) and D-7 (cross-domain canary) both produce per-arm
pass/fail signals BAI consumes via ingest_eval_result. D-8 (archive verify CLI)
surfaces the BAI promotion lineage to user.
