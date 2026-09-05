# B-23: Phase 4H — Self-Modification (DGM/CEL/DarwinNet-derived)
Objective: implement Phase 4H per spec — 3 layers (data, config,
structural); DGM sigmoid parent selection; staged eval (10/50/200);
TestOutcomeReport 4-category SWE-bench grading; MSI exp-decay per
PipelineMode; AllowedConfigKey enum; 4 enforcement layers.
Estimated effort: 3-4 sessions.
Pre-requisites: B-22 (Autonomous mode — self-mod runs autonomously),
B-15..B-20 (hardening).
Parallel-safe with: B-24, B-25, B-26, B-27, B-28.

## Context

Per spec phase-4H.txt and phase-4-summary.md PHASE 4H (13 sub-steps:
4H.1, 4H.2, 4H.3, 4H.3B, 4H.4, 4H.5, 4H.6, 4H.7, 4H.7B, 4H.7C, 4H.7D,
4H.8, 4H.9):

3 layers:
- L1 automatic metrics (no LLM): template rankings, rule trigger counts,
  pipeline outcome metrics. Pure arithmetic. Security/safety rules never
  touched.
- L1A automatic data-table growth (no LLM, no staged eval): additive rows
  in deprecated_patterns, FailureTaxonomy expansion, lazy-code regex,
  source-tier updates. FPR validation. DarwinNet Gene Pool stores
  successful patterns for reuse.
- L2 config proposals (DGM archive + staged eval + log-driven):
  sigmoid-weighted parent selection (lambda=10, alpha_0=0.5),
  staged evaluation (10/50/200 tasks), TestOutcomeReport (4-category
  SWE-bench grading, PASS_TO_FAIL → immediate reject)
- L3 structural (human-only, stubs in this phase)

DGM-derived: config archive max 50 variants per PipelineMode,
deceptive-dip resilient.

MSI (Modification Solidification Index): exponential-decay weighted
moving avg (lambda=0.3) of delta_score over N=10. Per-PipelineMode
(SURGICAL/GREENFIELD tracked independently).
- IMPROVING (>0.3) — continue
- STABILIZING (-0.1 to 0.3) — reduce frequency
- DIVERGING (<=-0.1) — pause + alert

Auto-revert on >10% degradation over 3 pipelines. Trigger task detects
objective hacking (DGM Appendix H).

AllowedConfigKey enum: closed set of mutable keys. Locked keys
(cloud budget, gate definitions, score weights) never modifiable.
Adding new keys requires Layer 3 human approval.

Safety: 4 enforcement layers (dispatch + IPC + filesystem + git).
Self-mod agent cannot read lock list. Cannot modify locked config keys.

Anti-staleness wiring: triggers nightly dep audit, Layer 1A grows
deprecated_patterns, triggers steering vector refresh.
Reasoning wiring: failure_taxonomy_misses → Layer 1A → new categories.

42+ tests (archive, proposals, staged eval, safety, MSI, adversarial).

## Reads

- F:\.ELAI_workflow\docs\specs\guide\steps\phase-4H.txt
- F:\.ELAI_workflow\docs\specs\integration\self-modification.txt
- summary/phase-4-summary.md PHASE 4H
- summary/integration-summary.md (self-modification section)

## Tasks

1. **L1 automatic metrics** (4H.1-4H.3): pure arithmetic over PipelineOutcome.
2. **L1A data-table growth** (4H.3B): wire new SsbActionType
   `data_table_grew` per growth event.
3. **L2 config archive** (4H.3-4H.5): DGM sigmoid-weighted parent selection;
   max 50 variants per PipelineMode; deceptive-dip resilient.
4. **Log-driven proposals** (4H.4): reads SSB + reasoning_trace; identifies
   WHERE reasoning failed; proposes config patch.
5. **Staged evaluation** (4H.5): 10/50/200 task progression; >40% pass + 
   top 2 promote to Stage 3.
6. **TestOutcomeReport** 4-category SWE-bench grading;
   PASS_TO_FAIL → immediate reject; SWR pre-screening optional.
7. **Safety: 4 enforcement layers** (4H.6): IPC + dispatch + filesystem
   + git locks. Self-mod agent cannot read lock list.
8. **MSI** (4H.7): exp-decay (lambda=0.3, N=10) per PipelineMode.
9. **AllowedConfigKey enum**: closed set mutable; locked keys (cloud_budget,
   gate definitions, score weights) never touched.
10. **Anti-staleness wiring**: trigger nightly dep audit, grow
    deprecated_patterns, refresh steering vector.
11. **Reasoning wiring**: failure_taxonomy_misses → L1A → new categories.
12. **L3 stubs**: structural changes (FSM states, agent roles, GRC protocol)
    HUMAN-ONLY; emit `l3_proposal` SSB row for human review.
13. **Dashboard surface**: ProposalsPanel (already built per 4K audit;
    activate behind feature flag) shows current archive, MSI per mode,
    pending proposals, staged eval results, last revert event.

14. **OSC -- Ontology-Similarity Convergence** (4H.7B, ouroboros port,
    MIT-licensed): `ConfigVariantSimilarity::compute(curr, prev) ->
    {key_overlap*0.5 + type_match*0.3 + exact_match*0.2}` over deserialized
    ConfigVariant.config_blob. Stagnant verdict at sim >= 0.95 over 3
    consecutive variants -> exploration boost. Oscillating verdict via
    PERIOD-2 FULL VERIFICATION (sim(N,N-2) AND sim(N-1,N-3) both >= 0.95)
    -> freeze the differing keys in AllowedConfigKey runtime exclusion
    list for 5 pipelines. Per-mode independence.
    Source: ouroboros src/ouroboros/core/lineage.py OntologyDelta and
    src/ouroboros/evolution/convergence.py _check_oscillation+_check_stagnation.

15. **REG -- Cross-Pipeline AC Regression** (4H.7C, ouroboros port):
    `RegressionDetector::detect(history) -> RegressionReport` spans the
    full PipelineOutcome history (TestOutcomeReport's PASS_TO_FAIL is
    single-eval; REG catches slow regressions across pipelines). Used as
    additional gate at 4H.5 promotion (block promotion despite score
    improvement) AND 4H.6 auto-revert trigger (3 consecutive regressed
    pipelines on active variant). Persistent failures (AC NEVER passed)
    are NOT regressions. Per-mode RegressionDetector.
    Source: ouroboros src/ouroboros/evolution/regression.py.

16. **EVO -- Evolution Gate** (4H.7D, ouroboros port): catches "fake
    IMPROVING" when MSI moves on score noise but config_blob is structurally
    identical. `evolved_count = count of (curr,prev) pairs with similarity
    < 0.95`. If `evolved_count == 0 AND latest_similarity >= 0.95` ->
    `Withhold` -> MSI verdict for that mode FORCED to STABILIZING.
    Source: ouroboros src/ouroboros/evolution/convergence.py:156-171
    _count_evolved_generations and the withholding clause.

17. **New SSB action_types** (4 added): `convergence_stagnant`,
    `config_oscillation`, `regression_detected`, `evolution_withheld`.
    Surfaced in MsiPanel (next to MSI verdict per mode, badge for
    STAGNANT/OSCILLATING/REGRESSED/WITHHELD).

## WIRING_CONTRACT

produces_symbols:
  - DefaultLayer1aProcessor
  - SelfModOrchestrator
  - ConfigArchive
  - evaluate_stage
  - run_full_pipeline
  - TestOutcomeReport
  - MsiTracker
  - AllowedConfigKey
  - is_locked
  - check_locked_key
  - should_auto_revert
  - ConfigVariantSimilarity
  - OscVerdict
  - detect
  - RegressionReport
  - compute_evolution_gate
  - EvolutionGateResult
  - SsbActionType::ProposalGenerated
  - SsbActionType::ProposalApplied
  - SsbActionType::ProposalRejected
  - SsbActionType::MsiUpdate
  - SsbActionType::DataTableGrew
  - SsbActionType::L3Proposal
  - SsbActionType::ConvergenceStagnant
  - SsbActionType::ConfigOscillation
  - SsbActionType::RegressionDetected
  - SsbActionType::EvolutionWithheld
consumes_symbols:
  - PipelineOutcome stream (existing)
  - ProposalsPanel.svelte (already built)
reachable_from:
  - autonomous mode loop (B-22)
  - nightly self-mod cron
stub_ban:
  - locked config keys must NEVER be modifiable by self-mod agent
  - PASS_TO_FAIL test outcome MUST trigger immediate reject
  - DIVERGING MSI MUST pause + alert
frontend_pair: ProposalsPanel + new MsiPanel
observable_effect:
  trigger: cd /d F:\.ELAI_workflow\.elai_cc && set ELAI_SSB_SMOKE_DB=F:\.ELAI_workflow\.elai_cc\sessions\ssb.db && cargo test --package elai-orchestrator self_mod_layer1a::tests::ssb_emission_smoke
  expect_ssb: msi_update
  expect_artifact: ProposalsPanel shows current archive (max 50 variants);
                   MsiPanel shows MSI per mode (IMPROVING / STABILIZING /
                   DIVERGING)

## DETERMINISTIC_CHECK

```bash
# 1. AllowedConfigKey enum
rg "enum AllowedConfigKey" .elai_cc/crates/orchestrator/src
# 2. 42 tests
cargo test --package elai-orchestrator self_mod 2>&1 | grep "test result"
# Expected: ≥42 tests.
```

## ACCEPTANCE

- [ ] All 13 sub-steps deliverables present
- [ ] L1/L1A/L2 layers functional
- [ ] DGM archive max 50 variants per mode
- [ ] Staged eval 10/50/200 working
- [ ] TestOutcomeReport 4 categories
- [ ] MSI per PipelineMode + DIVERGING pause
- [ ] AllowedConfigKey closes set; locked keys safe
- [ ] 4-layer safety locks
- [ ] Anti-staleness + reasoning wirings active
- [ ] L3 stubs emit `l3_proposal` only (no auto-promote)
- [ ] ProposalsPanel + MsiPanel surface real state
- [ ] OSC: stagnation + period-2-full oscillation per mode (4H.7B)
- [ ] REG: cross-pipeline AC regression blocks promotion + triggers
      auto-revert (4H.7C)
- [ ] EVO: evolution gate forces MSI=STABILIZING when config_blob
      doesn't actually evolve (4H.7D)
- [ ] 4 new SSB action_types live (convergence_stagnant,
      config_oscillation, regression_detected, evolution_withheld)
- [ ] 54+ tests pass (was 42 in pre-OSC/REG/EVO scope)

## Files touched

- ADD: crates/orchestrator/src/self_mod.rs (L1/L1A/L2)
- ADD: crates/orchestrator/src/config_archive.rs
- ADD: crates/orchestrator/src/msi.rs
- ADD: crates/orchestrator/src/osc.rs (ConfigVariantSimilarity + OscVerdict)
- ADD: crates/orchestrator/src/regression.rs (RegressionDetector port)
- ADD: crates/orchestrator/src/evolution_gate.rs (EVO port)
- MODIFY: crates/orchestrator/src/ssb.rs (11 new action_types: 7 base
  self-mod + convergence_stagnant + config_oscillation +
  regression_detected + evolution_withheld)
- MODIFY: crates/dashboard-v2/src-tauri/src/lib.rs (V2 — was V1
  crates/dashboard/src/commands.rs); ADD crates/dashboard-v2/src/lib/
  components/{ProposalsPanel,MsiPanel}.svelte (V2 has no proposals/ or
  msi/ subdirectory). Stub.ts already exposes get_pending_proposals,
  get_applied_proposals, apply_proposal_from_user, reject_proposal,
  get_msi_status — keep names 1:1 when wiring
- ADD: scripts/self_mod_cron.sh (optional nightly)
- CREATE: workflow/reasoning/B-23-decisions.md

## Next

B-24 (4I Media Validator).
