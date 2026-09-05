# D-29: Hierarchical BAI for non-IID rewards (DEFERRED — research-grade)

Objective: published BAI (D-5; Audibert COLT 2010) assumes IID rewards.
ELAI pipeline outcomes are NOT IID across sessions (different tasks,
different priors). Hierarchical Bayesian BAI may close this gap. Closure
includes "infeasibility note" as valid outcome (companion §3.7 Q4 lines
421-422).
Estimated effort: 4-8 sessions wall-clock + RunPod for Gibbs sampler.
Status: DEFERRED.
Pre-requisites: D-5 (flat BAI baseline measured for comparison). RunPod for
  hierarchical Gibbs sampling.
Parallel-safe with: D-28, D-30 (independent deferred).

## Context

Companion §3.7 + Q4 (lines 296-317 + 421-422). Hierarchical models in BAI:
- Per-arm: empirical pass/fail.
- Per-family: hyperprior over per-arm pass-rates.
- Hierarchical posterior: pool information across arms within same family.

Per Variance-Dependent BAI (arxiv:2106.10417, T1) + hierarchical Bayesian
literature (out of immediate D-29 scope to verify), hierarchical models can
reduce regret on non-IID arms by 30-50% at the cost of compute.

Risk: hierarchical model may not improve on flat BAI for ELAI's specific
non-IID structure. Closure with "infeasibility note + concrete blocker"
is valid outcome.

## Reads

- F:\.ELAI_workflow\docs\specs\guide\reasoning-improvement-grounded-2026-05-19.md
  (§3.7 BAI; Q4 hierarchical — lines 296-317 + 421-422)
- F:\.ELAI_workflow\workflow\papers\audibert-colt-2010-bai-interface.md (D-5)
- F:\.ELAI_workflow\workflow\papers\variance-bai-2106.10417-interface.md (D-5)
- .elai_cc/crates/orchestrator/src/bai.rs (D-5 flat baseline)

## Paper-ingestor pre-step

Spawn paper-ingestor with mode=paper for:
- Hierarchical Bayesian bandit literature (Gelman, Carlin et al "Bayesian
  Data Analysis" book ch. 7; or recent arxiv per literature search at
  D-29 unblock time).

Verify workflow/papers/{hierarchical-bandit-<ref>}-interface.md.

## Tasks

1. **Hierarchical model spec** at .elai_cc/python/bai_hierarchical/:
   - Per-arm: Beta(alpha_i, beta_i).
   - Hyperprior on (alpha_i, beta_i): Gamma(a, b) (or similar).
   - Posterior inference: Gibbs sampling (CPU-bound; small problem space —
     ~50 arms × 200 evals).

2. **Synthetic non-IID benchmark**:
   - 5-family bandit, 10 arms per family.
   - Within-family arms correlated; across-family independent.
   - Compare flat BAI vs hierarchical regret over 1000 pulls.

3. **Closure paths**:
   - Path A: hierarchical regret ≤ flat-BAI regret on synthetic; productize.
   - Path B: hierarchical equal or worse on synthetic; document infeasibility
     note with ≥1 concrete blocker (e.g. compute overhead not justified;
     non-IID structure not as strong as hypothesized).

4. **If Path A**: integrate hierarchical solver into D-5 BAI (gated by
   ELAI_BAI_MODE=hierarchical env var; default flat).

5. **Decision doc** workflow/reasoning/D-29-decisions.md (DEFERRED — drafted
   at unblock).

## WIRING_CONTRACT

produces_symbols:
  - HierarchicalBaiState (bai_hierarchical.rs:271) + FamilyHyperprior (bai_hierarchical.rs:202)
  - FamilyId (bai_hierarchical.rs:115) + SelectionBasis enum (bai_hierarchical.rs:246)
  - hierarchical_update (bai_hierarchical.rs:509) + hierarchical_arm_select (bai_hierarchical.rs:605)
  - total_variance_decomposition (bai_hierarchical.rs:659)
  - family_id_for_domain + family_id_for_template_or_mixed (bai_hierarchical.rs:159,183)
  - SsbActionType::HierarchicalBaiRoundCompleted + HierarchicalBaiRoundCompletedPayload (ssb.rs:6725)
  - BaiMode::Hierarchical (runtime_config.rs:440) gated by ELAI_BAI_MODE=hierarchical
  # NOTE: Path A (productize) shipped; Path B (infeasibility) was not the outcome.
  # Python bai_hierarchical/ solver is DEFERRED/RUNPOD-PENDING (Gibbs sampler benchmark).
consumes_symbols:
  - D-5 darwinian_bai flat baseline (prerequisite for comparison; darwinian_bai::DEFAULT_DISCOUNT_FACTOR)
  - BaiMode::from_env (runtime_config.rs:449) — read from ELAI_BAI_MODE env var
  - TemplateId + Domain (reasoning_template) for family assignment
  - SsbWriter
reachable_from:
  - task_runner.rs:11406 (BaiMode::Hierarchical branch in promotion dispatch loop)
  - ELAI_BAI_MODE=hierarchical + ELAI_BAI_ENABLE=1 env vars (wire-dark default: flat)
  # RUNPOD-PENDING: Gibbs sampling synthetic benchmark (5-family, 10-arm, 1000-pull
  # regret comparison) requires RunPod CPU instance; not yet run.
stub_ban:
  - hierarchical_arm_select MUST use SelectionBasis; not fall through to flat
  - HierarchicalBaiRoundCompleted SSB row MUST be emitted per round (not SKIP)
  - BaiMode gate MUST default Flat when ELAI_BAI_MODE unset (runtime_config.rs:454 confirmed)
observable_effect:
  trigger: ELAI_BAI_MODE=hierarchical ELAI_BAI_ENABLE=1 elai task <any_task>
  expect_ssb: 1 HierarchicalBaiRoundCompleted row with gibbs_iterations > 0
  expect_artifact: SelectionBasis arm selection recorded in SSB payload
  # RUNPOD-PENDING: synthetic regret benchmark (flat vs hierarchical delta) is the
  # research-closure gate; requires RunPod CPU instance per D-29 tasks.
  # The Rust host-side dispatch (task_runner.rs:11406) IS wired and live.

## DETERMINISTIC_CHECK

DEFERRED — populated at D-29 unblock time.

## ACCEPTANCE

- [ ] DEFERRED. Sessions unblock when: D-5 has stable flat-BAI baseline.
- [ ] RunPod CPU instance available.
- [ ] Hierarchical-bandit paper-ingestor output present.
- [ ] Synthetic 5-family benchmark report (flat vs hierarchical regret).
- [ ] Either Path A (productize) OR Path B (infeasibility note with
      concrete blocker).
- [ ] Decision doc workflow/reasoning/D-29-decisions.md.

## Files touched (anticipated)

- ADD: .elai_cc/python/bai_hierarchical/ (Python hierarchical solver)
- ADD: .elai_cc/crates/orchestrator/src/bai_hierarchical.rs (if Path A)
- ADD: workflow/reasoning/D-29-decisions.md

## Out-of-scope

- Multi-level hierarchical (DEFERRED beyond D-29; 2-level suffices)
- Online hierarchical updates (DEFERRED)

## Next

Independent of D-28 and D-30.
