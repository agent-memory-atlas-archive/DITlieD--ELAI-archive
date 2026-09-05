# D-16: Conformal refusal calibration (UniCR) — formal Mode E grounding

Objective: implement UniCR-style conformal risk control on ELAI's
i-don't-know outputs. Distribution-free FORMAL guarantee that refusal rate
stays within user-specified ε regardless of distribution shift (UniCR
arxiv:2509.01455, T1). This is the ONLY Mode E grounding with mathematical
guarantee. Real-problem §4.5 (lines 274-286): "the strongest math available
for 'i don't know.'"
Estimated effort: 3-4 sessions.
Pre-requisites: D-9 (L3 frozen-surface enforcement; τ updater MUST be L3 —
  agent cannot bias calibration), D-10 (Mode E target slot exists).
  spec-grounder verifies.
Parallel-safe with: D-17 (SLA monitoring), D-22 (calibration validation
  benchmark gates D-16).

## Context

Real-problem §4.5 (lines 274-286). UniCR (arxiv:2509.01455, T1) provides
distribution-free conformal risk control:
- Maintain calibration set: tuples (claim, ground_truth_available,
  model_confidence) at .elai_cc/calibration/.
- Compute threshold τ such that empirical risk (refusing-when-truth-exists
  AND not-refusing-when-no-truth) stays within user-specified ε. Default
  ε = 0.1.
- τ is updated nightly from new calibration data (L3 — system cannot
  self-modify the calibration loop).
- System REFUSES any claim whose model confidence (composite of GG signals:
  mean_logprob, entropy, novelty) falls below τ.

Other Mode E refusal mechanisms (calibration-only, prompt-only) lack the
formal guarantee.

Partial foundation present and usable:
- D-3 IntrinsicSignals struct provides composite confidence input.
- D-9 L3 enforcement: locked_keys_registry.toml entry for tau-updater
  prevents agent tampering.
- D-10 GroundingMode::Refuse target (already exists).

## Reads

- F:\.ELAI_workflow\docs\specs\guide\real-problem-solving-grounded-2026-05-19.md
  (§4.5 UniCR — lines 274-286; §1 Mode E — lines 105-118)
- .elai_cc/crates/orchestrator/src/verification_result.rs (D-3 IntrinsicSignals)
- .elai_cc/crates/orchestrator/src/grounding_mode.rs (D-10 GroundingMode::Refuse)
- .elai_cc/scripts/locked_keys_registry.toml (D-9; add τ-updater entry)

## Paper-ingestor pre-step

Spawn paper-ingestor with mode=paper for:
- arxiv:2509.01455 UniCR Trusted Uncertainty
- arxiv:2311.09731 Uncertainty Outside Parametric Knowledge
- arxiv:2503.02623 Rewarding Doubt (RL-trained calibrated confidence)

Verify workflow/papers/{unicr-2509.01455,uncertainty-outside-parametric-
2311.09731,rewarding-doubt-2503.02623}-interface.md.

## Tasks

1. **CalibrationTuple types** in new file
   .elai_cc/crates/orchestrator/src/conformal_calibration.rs:
   - `struct CalibrationTuple { claim_id: ClaimId, ground_truth_available:
     bool, model_confidence: f32, claim_type: ClaimType, claim_outcome:
     ClaimOutcome, recorded_at: DateTime<Utc> }`
   - `enum ClaimOutcome { Verified, Falsified, Refused, NoCheck }`
   - `struct CalibrationCorpus { tuples: Vec<CalibrationTuple>,
     storage_path: PathBuf }`
   - `impl CalibrationCorpus`:
     - `load(path)`: walk path/*.toml; parse each as CalibrationTuple.
     - `append(tuple)`: write tuple to path/{claim_id}.toml.
     - `size() -> usize`

2. **τ computation** in conformal_calibration.rs (UniCR §3):
   - `fn compute_tau(corpus: &CalibrationCorpus, epsilon: f32) -> Result<f32,
     ConformalError>`:
     - Per UniCR §3.2 conformal prediction with risk control.
     - Compute empirical risk function r(τ) = P(refuse | truth_exists) +
       P(accept | truth_doesnt_exist) over corpus.
     - Find minimal τ such that r(τ) ≤ ε. Use binary search over candidates
       in [0, 1] with 0.01 grid.
     - Return τ or InsufficientData error if corpus.size() < 500.
   - Test: synthetic corpus with known split → τ matches hand-computed.

3. **TauUpdater (nightly, L3-only)** in conformal_calibration.rs:
   - `struct TauUpdater { storage_path: PathBuf, epsilon: f32, current_tau:
     f32 }`
   - `fn update_nightly(&mut self) -> Result<f32, ConformalError>`:
     - Load CalibrationCorpus.
     - Compute new τ.
     - Persist to .elai_cc/calibration/tau.txt with timestamp.
     - Emit CalibrationUpdate SSB row.
   - L3-LOCKED: this function is in locked_keys_registry.toml; agent
     cannot edit.

4. **Refusal decision** in conformal_calibration.rs:
   - `fn refusal_decision(signals: &IntrinsicSignals, tau: f32) -> bool`:
     compute composite confidence = w1*signals.mean_logprob + w2*(-signals.
     entropy_max) + w3*signals.novelty (weights from elai_cc.toml,
     L3-LOCKED). Return true if composite < τ.
   - Test: composite scoring deterministic; threshold logic correct.

5. **GroundingMode integration** in
   .elai_cc/crates/orchestrator/src/grounding_mode.rs (D-10):
   - select_grounding_mode internal claim.intrinsic_signals_below_threshold
     wraps refusal_decision(signals, ctx.unicr_tau).
   - ctx.unicr_tau loaded from .elai_cc/calibration/tau.txt at startup.

6. **Background calibration corpus growth**:
   - Hook into task_runner.rs post-claim-resolution: append CalibrationTuple
     to corpus (claim_id, ground_truth_available = based on whether check
     produced result, model_confidence = composite at decision time).
   - Append is L1A-style (data-table only); no LLM judgment in the loop.

7. **CLI subcommand `elai calibration status`**:
   - In .elai_cc/crates/cli/src/calibration_status.rs.
   - Print: corpus.size, current τ, last update timestamp, ε threshold,
     empirical refusal rate over last N sessions.
   - Exit 0 always; informational.

8. **CLI subcommand `elai calibration update`**:
   - In same file.
   - Manual trigger for TauUpdater.update_nightly (operator may invoke when
     refusal rate seems off-band).
   - L3-protected: requires L3_OVERRIDE:calibration_update commit-message tag
     when invoked via CI; allowed freely for dev sessions.

9. **SSB action types** in ssb.rs (SCHEMA_VERSION 61→62):
   - `RefusalDecision { claim_id: ClaimId, composite_confidence: f64,
     tau: f64, refused: bool }`
   - `CalibrationUpdate { tuple_count: u32, new_tau: f64, old_tau: f64,
     epsilon: f64, audit_id: Uuid }`
   - Round-trip tests.

10. **elai_cc.toml [conformal] section**:
    - `[conformal]\nepsilon = 0.1\nmin_corpus_size = 500\n
      composite_weights = { mean_logprob = 0.4, entropy = -0.3, novelty = 0.3 }\n
      tau_storage_path = ".elai_cc/calibration/tau.txt"\n
      corpus_storage_path = ".elai_cc/calibration/corpus/"`
    - L3-LOCKED.

11. **Unit tests** in conformal_calibration.rs:
    - compute_tau on synthetic corpus with known split → τ matches.
    - compute_tau on corpus.size() < 500 → InsufficientData.
    - refusal_decision threshold logic table (≥8 scenarios).
    - TauUpdater.update_nightly idempotency (same corpus → same τ).

12. **Integration test** in
    .elai_cc/crates/orchestrator/tests/conformal_integration.rs:
    - Synthesize 600-tuple calibration corpus with known ground-truth-
      available distribution.
    - Compute τ; assert empirical refusal rate within [τ-ε, τ+ε].

13. **Decision doc** workflow/reasoning/D-16-decisions.md (3 decisions):
    - (a) Why ε=0.1 default: arxiv:2509.01455 §6 — typical operating point;
      tunable per project.
    - (b) Why min_corpus_size=500: UniCR §3.4 — finite-sample bound requires
      N ≥ ~500 for ε=0.1 to hold with high probability.
    - (c) Why composite_weights for confidence (not raw mean_logprob): single-
      signal under-uses available evidence; composite per GG-signal block
      (mean_logprob, entropy, novelty) is a more robust input.

## WIRING_CONTRACT

produces_symbols:
  - CalibrationTuple + CalibrationCorpus + ClaimOutcome (closed enum)
  - TauUpdater + compute_tau + refusal_decision
  - SsbActionType::RefusalDecision + ::CalibrationUpdate + payloads
  - SCHEMA_VERSION bump 61→62
  - [conformal] elai_cc.toml section
  - CLI `elai calibration status` + `elai calibration update`
  - .elai_cc/calibration/{tau.txt, corpus/} layout
consumes_symbols:
  - IntrinsicSignals (D-3)
  - GroundingMode dispatch (D-10)
  - SsbWriter
  - L3 locked-keys registry (D-9; TauUpdater is L3-locked)
reachable_from:
  - GroundingMode::Refuse decision point
  - Nightly TauUpdater (cron job; CI-scheduled)
  - `elai calibration status/update` CLI
stub_ban:
  - compute_tau MUST require corpus.size() >= min_corpus_size; not approximate
    on small samples
  - TauUpdater MUST emit CalibrationUpdate SSB row (audit trail)
  - refusal_decision MUST use weighted composite, not raw signal (per
    real-problem §4.5)
  - tau.txt MUST be written by TauUpdater alone; LockedKeysAuditor protects
observable_effect:
  trigger: synthesize 600-tuple corpus; run TauUpdater.update_nightly
  expect_ssb: 1 CalibrationUpdate row with new_tau computed; subsequent
              cloud /task with low-confidence claim → RefusalDecision row
              with refused=true
  expect_artifact: .elai_cc/calibration/tau.txt written; corpus path has
                   ≥600 tuple files

## DETERMINISTIC_CHECK

```bash
rg "SCHEMA_VERSION: i64 = 62" .elai_cc/crates/orchestrator/src/ssb.rs
rg "RefusalDecision|CalibrationUpdate" .elai_cc/crates/orchestrator/src/ssb.rs
rg "struct TauUpdater|fn compute_tau|fn refusal_decision" \
  .elai_cc/crates/orchestrator/src/conformal_calibration.rs
rg "\[conformal\]" .elai_cc/elai_cc.toml
test -f .elai_cc/scripts/locked_keys_registry.toml && \
  rg "TauUpdater\|compute_tau\|tau.txt" .elai_cc/scripts/locked_keys_registry.toml
cargo check --workspace 2>&1 | tail -3
cargo test -p elai_orchestrator -- conformal 2>&1 | grep "test result"
test -f workflow/papers/unicr-2509.01455-interface.md && \
  test -f workflow/papers/uncertainty-outside-parametric-2311.09731-interface.md && \
  test -f workflow/papers/rewarding-doubt-2503.02623-interface.md && echo OK
```

## ACCEPTANCE

- [ ] SCHEMA_VERSION_PUBLIC == 62
- [ ] 2 SSB variants round-trip
- [ ] CalibrationCorpus load/append on tmpdir
- [ ] compute_tau correct on synthetic corpus with known split
- [ ] compute_tau returns InsufficientData on < 500 tuples
- [ ] refusal_decision threshold logic table (≥8 scenarios) correct
- [ ] TauUpdater.update_nightly idempotent on identical corpus
- [ ] L3 lock registered for TauUpdater + compute_tau + tau.txt
- [ ] elai_cc.toml [conformal] loaded
- [ ] cargo check --workspace: zero new errors
- [ ] cargo test -p elai_orchestrator -- conformal: all pass, ≥12 tests
- [ ] Cloud smoke test A: low-confidence synthetic claim → Mode E dispatch →
      RefusalDecision row with refused=true
- [ ] Cloud smoke test B: high-confidence synthetic claim → Mode E NOT
      dispatched (other mode chosen); no RefusalDecision row in that branch
- [ ] Cloud smoke test C: manual `elai calibration update` (with L3_OVERRIDE
      tag) → CalibrationUpdate SSB row + tau.txt updated
- [ ] All 3 paper-ingestor outputs present
- [ ] Decision doc workflow/reasoning/D-16-decisions.md

## Files touched

- ADD: .elai_cc/crates/orchestrator/src/conformal_calibration.rs
- ADD: .elai_cc/crates/cli/src/calibration_status.rs
- MODIFY: .elai_cc/crates/orchestrator/src/ssb.rs (SCHEMA_VERSION 61→62;
    2 new variants)
- MODIFY: .elai_cc/crates/orchestrator/src/grounding_mode.rs (D-10
    intrinsic_signals_below_threshold integration)
- MODIFY: .elai_cc/crates/orchestrator/src/task_runner.rs (post-claim corpus
    growth hook)
- MODIFY: .elai_cc/elai_cc.toml ([conformal] section)
- MODIFY: .elai_cc/scripts/locked_keys_registry.toml (TauUpdater L3-lock)
- MODIFY: .elai_cc/crates/cli/src/commands.rs (SlashCommand::CalibrationStatus
    + CalibrationUpdate)
- MODIFY: .elai_cc/crates/orchestrator/src/lib.rs (pub mod conformal_calibration)
- ADD: .elai_cc/crates/orchestrator/tests/conformal_integration.rs
- ADD: workflow/reasoning/D-16-decisions.md
- ADD: workflow/papers/{unicr-2509.01455,uncertainty-outside-parametric-
       2311.09731,rewarding-doubt-2503.02623}-interface.md

## Out-of-scope

- D-22 calibration validation under distribution shift (benchmarks D-16)
- D-30 calibration bootstrap from generic corpus (DEFERRED; needs N=500
  before D-16 has empirical signal)
- Multi-domain τ (per-family τ; DEFERRED to D-30 if needed)

## Next

D-17 (honest-refusal SLA) consumes RefusalDecision rate as session metric.
D-22 validates distribution-shift behavior.
