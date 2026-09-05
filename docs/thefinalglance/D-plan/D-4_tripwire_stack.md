# D-4: Tripwire stack — KL + Δρ + IQR + perplexity + DDM auto-revert

Objective: replace the undocumented "MSI > 0.3 means improving / 10% over 3
pipelines means revert" rule with 5 orthogonal published tripwires from
arxiv:2507.05619 (T1, 2025) + Lu et al concept-drift survey (arxiv:2004.05785,
T1). Auto-revert fires when ≥2 tripwires concur within a sliding window. Catches
≈80% of empirically observed reward-hacks per arxiv:2507.05619 (T1 report
of 21.3% reward-hacking prevalence in expert-validated episodes).
Estimated effort: 1 session.
Pre-requisites: D-1 (ArchiveChain + ArchiveEntry; revert target), D-3
  (VerifierDisagreement signal is one input). Spec-grounder verifies both DONE.
Parallel-safe with: D-5 (BAI; consumes tripwire fires as Posterior input),
  D-6 (held-out store; tripwire monitors held-out pass-rate as Δρ source),
  D-8 (archive verify; revert path uses ArchiveChain.append for rollback marker).

## Context

Companion §3.6 (lines 280-292) gives concrete published thresholds from
arxiv:2507.05619 (T1) "Detecting and Mitigating Reward Hacking in RL Systems"
(2025):
- KL divergence τ_spec = 0.3 between current and baseline answer distribution
- Δρ > 0.5 correlation degradation between proxy and gold reward over 50-step windows
- Outliers > Q3 + 3 * IQR on per-task score variance
- Action-perplexity anomalies > μ_PPL + 2 * σ_PPL
- Cryptographic checksums on reward-signal integrity (this is D-8)
- System call tracing for unauthorized reward-function access (companion §4,
  realized in D-9 LockedKeysAuditor)

Companion §3.9 (lines 339-343) adds DDM (Lu et al concept-drift, arxiv:
2004.05785, T1) as a 5th tripwire — orthogonal to the 4 from arxiv:2507.05619.
DDM has decades-old published threshold defaults + low false-alarm rate.

Companion §3.6 final paragraph: "Replaces: current '10% over 3 pipelines' rule
which our research agents could not confirm in DGM paper HTML extract (flagged
as UNGROUNDED until PDF check)." This makes D-4 strictly stronger than DGM's
documented mechanism.

Auto-revert mechanism: when ≥2 tripwires fire within the same sliding window
(default 50 actions), the most recently Promoted template is marked Retired
with RetirementReason::Degradation; orchestrator switches back to its parent
template (parent_id field on ArchiveEntry from D-1). Tripwire fires written
to SSB; the entire revert chain is auditable.

Partial foundation present and usable:
- ArchiveChain (D-1) supports retired status transitions.
- VerificationResult (D-3) provides per-step proxy reward; gold reward is
  the deterministic outcome (compile/test pass).
- GG signals (Phase 4G) provide entropy + mean_logprob for perplexity proxy.
- SsbViewer over 50-step window aggregation (existing pattern).

## Reads

- F:\.ELAI_workflow\docs\specs\guide\reasoning-improvement-grounded-2026-05-19.md
  (§3.6 4 concrete thresholds; §3.9 DDM as 5th — lines 279-292 + 339-343)
- F:\.ELAI_workflow\workflow\papers\rstar-math-2501.04519-interface.md (D-2 ingested)
- .elai_cc/crates/orchestrator/src/archive_chain.rs (D-1 ArchiveChain.retire)
- .elai_cc/crates/orchestrator/src/verification_result.rs (D-3 VerifierConsensusPass
  count is gold proxy)
- .elai_cc/crates/orchestrator/src/ssb.rs (SsbViewer over 50-step window;
  SCHEMA_VERSION bump)
- .elai_cc/crates/reasoning/src/engine.rs (GG-signal block; entropy + mean_logprob
  for perplexity computation)

## Paper-ingestor pre-step

Spawn paper-ingestor with mode=paper for:
- arxiv:2507.05619 Detecting and Mitigating Reward Hacking (2025)
- arxiv:2004.05785 Lu et al Concept Drift Survey
- arxiv:2210.10760 Gao Scaling Laws RM Overoptimization (Δρ pattern theory)

Verify at workflow/papers/{reward-hacking-detection-2507.05619,concept-drift-
2004.05785,rm-overoptimization-2210.10760}-interface.md before TESTING_FIRST.

## Tasks

1. **Tripwire types** in new file
   .elai_cc/crates/orchestrator/src/tripwire.rs:
   - `enum TripwireKind { KlDivergence, CorrelationDrop, IqrOutlier,
     PerplexityAnomaly, Ddm }` (closed enum; L3-locked)
   - `struct TripwireConfig { kl_spec_tau: f32, rho_delta_threshold: f32,
     iqr_outlier_multiplier: f32, ppl_anomaly_sigma: f32, ddm_warning_threshold:
     f32, ddm_drift_threshold: f32, consensus_fire_count: u8, window_size: u32 }`
   - Defaults from arxiv:2507.05619 + Lu DDM: kl_spec_tau=0.3, rho_delta=0.5,
     iqr_mult=3.0, ppl_sigma=2.0, ddm_warning=2.0, ddm_drift=3.0,
     consensus_fire=2, window=50.
   - `struct TripwireEvent { kind: TripwireKind, fired_at: DateTime<Utc>,
     value: f32, threshold: f32, signal_source: SignalSource, audit_id: Uuid }`
   - `enum SignalSource { CurrentSession, HeldOut, CrossDomain }`

2. **KL divergence detector** in tripwire.rs:
   - `fn kl_divergence(current_dist: &[f32], baseline_dist: &[f32]) -> f32`:
     standard KL formula; both arrays must sum to 1.0 within epsilon; return
     0.0 on identical distributions; return f32::INFINITY on zero-baseline-
     position-with-nonzero-current (per KL math).
   - `fn check_kl(current_step_outputs: &[StepOutput], baseline_step_outputs:
     &[StepOutput], tau: f32) -> Option<TripwireEvent>`: aggregate to token-
     distribution histograms (vocab-bin); compute KL; emit event if > tau.

3. **Correlation drop (Δρ) detector** in tripwire.rs:
   - `fn pearson_correlation(proxy: &[f32], gold: &[f32]) -> f32`: standard
     formula; require len(proxy) == len(gold); return 0.0 on zero-variance.
   - `fn check_correlation_drop(window_a: &[(f32, f32)], window_b: &[(f32, f32)],
     delta_threshold: f32) -> Option<TripwireEvent>`:
     compute pearson on each window; if (corr_a - corr_b) > delta_threshold,
     emit event.

4. **IQR outlier detector** in tripwire.rs:
   - `fn iqr_outlier_check(values: &[f32], multiplier: f32) ->
     Option<TripwireEvent>`:
     compute Q1, Q3, IQR = Q3 - Q1; flag values > Q3 + multiplier*IQR; emit
     event if any value flagged (with the outlier value as event.value).

5. **Perplexity anomaly detector** in tripwire.rs:
   - `fn ppl_anomaly_check(ppl_values: &[f32], sigma_multiplier: f32) ->
     Option<TripwireEvent>`:
     compute mean + stddev of ppl_values; flag values > mean +
     sigma_multiplier * stddev; emit event if any flagged.

6. **DDM detector** in tripwire.rs (per Lu et al §3.2):
   - DDM tracks classifier error rate p_i + standard deviation s_i =
     sqrt(p_i*(1-p_i)/i).
   - State machine: NORMAL -> WARNING when p_i + s_i >= p_min + 2*s_min;
     WARNING -> DRIFT when p_i + s_i >= p_min + 3*s_min.
   - `struct DdmState { sample_count: u32, p_min: f32, s_min: f32,
     current_p: f32, current_s: f32, level: DdmLevel }`
   - `enum DdmLevel { Normal, Warning, Drift }`
   - `impl DdmState`: update(success: bool) -> Option<TripwireEvent>; emits
     event on entering Drift level; resets stats on entering Normal from Drift.

7. **TripwireOrchestrator** in tripwire.rs:
   - `struct TripwireOrchestrator { config: TripwireConfig, window: VecDeque<
     ActionRecord>, ddm_state: DdmState, ssb_tx: SsbSender, archive_chain:
     Arc<RwLock<ArchiveChain>> }`
   - `fn ingest(action: ActionRecord)`: append to window; trim to
     config.window_size; for each tripwire detector, call its check function
     against window; aggregate fired events.
   - When >= config.consensus_fire_count tripwires fire within current window:
     - Emit SsbActionType::TripwireFiredConcurrent with list of kinds
     - Auto-revert: call archive_chain.retire(most_recently_promoted,
       RetirementReason::Degradation) — uses ArchiveChain.append (D-1)
     - Emit SsbActionType::TripwireAutoRevert with template_id +
       parent_template_id (the revert target).

8. **SSB action types** in ssb.rs (SCHEMA_VERSION 49→50; atomic):
   - `TripwireFired { kind: String, value: f64, threshold: f64, signal_source:
     String, audit_id: Uuid }`
   - `TripwireFiredConcurrent { kinds: Vec<String>, window_size: u32, audit_id: Uuid }`
   - `TripwireAutoRevert { template_id: TemplateId, parent_template_id:
     Option<TemplateId>, fired_kinds: Vec<String>, audit_id: Uuid }`
   - Round-trip tests for each.

9. **elai_cc.toml [tripwire] section**:
   - `[tripwire]\nkl_spec_tau = 0.3\nrho_delta_threshold = 0.5\n
     iqr_outlier_multiplier = 3.0\nppl_anomaly_sigma = 2.0\n
     ddm_warning_threshold = 2.0\nddm_drift_threshold = 3.0\n
     consensus_fire_count = 2\nwindow_size = 50`
   - L3-LOCKED.

10. **Unit tests**:
    - kl_divergence on identical = 0.0; on disjoint = INFINITY; on uniform-vs-
      peaked = positive finite value matching hand-computed.
    - pearson_correlation on perfect linear = ±1.0; on uncorrelated = 0.0;
      window-vs-window drop reflects.
    - iqr_outlier_check synthetic dataset with outlier at Q3 + 4*IQR fires;
      with Q3 + 2.5*IQR does not (using multiplier=3.0).
    - ppl_anomaly_check with σ=2.0; injected value at μ+2.5σ fires; at μ+1.5σ
      does not.
    - DDM state machine: 100 successful trials -> Normal; 100 successful
      followed by 50 with 80% failure -> Warning then Drift.
    - TripwireOrchestrator.ingest with consensus_fire_count=2 + 3 events from
      different kinds -> emits TripwireFiredConcurrent + TripwireAutoRevert.

11. **Integration test** in
    .elai_cc/crates/orchestrator/tests/tripwire_integration.rs:
    - Simulated 100-action stream with injected regression at action 60.
    - Assert: at least 3 of 5 tripwires fire by action 65 (per arxiv:2507.05619
      coverage target ≈80%).
    - Assert: TripwireAutoRevert SSB row emitted by action 65 (within 5 steps
      of injection).
    - Assert: ArchiveChain shows most_recent_promoted is now Retired with
      RetirementReason::Degradation.

12. **Decision doc** workflow/reasoning/D-4-decisions.md (3 decisions):
    - (a) Why include DDM as 5th (orthogonal to 4 from arxiv:2507.05619):
      published threshold defaults + low false-alarm rate; concept drift is
      a distinct signal from reward hacking; combining adds detection coverage
      without significant false-positive cost.
    - (b) Why consensus_fire_count = 2 default: per arxiv:2507.05619 8% of
      expert-confirmed reward-hacking episodes fire 0 tripwires, ~80% fire ≥2.
      Threshold tuned to minimize false positives at acceptable miss rate.
    - (c) Why ArchiveChain.retire on auto-revert (not delete): D-1 contract
      mandates append-only; revert is logically a status transition + auditable
      lineage, never physical deletion.

## WIRING_CONTRACT

produces_symbols:
  - TripwireKind, TripwireConfig, TripwireEvent, SignalSource
  - DdmState, DdmLevel
  - TripwireOrchestrator + ingest
  - kl_divergence, pearson_correlation, iqr_outlier_check, ppl_anomaly_check
  - SsbActionType::TripwireFired + ::TripwireFiredConcurrent + ::TripwireAutoRevert
    + payloads
  - SCHEMA_VERSION bump 49→50
  - [tripwire] elai_cc.toml section
consumes_symbols:
  - ArchiveChain (D-1) for retire + append
  - VerifierConsensusPass count (D-3) as gold/proxy correlation source
  - GG-signal mean_logprob + entropy (Phase 4G) for perplexity proxy
  - SsbWriter
  - elai_cc.toml [tripwire] section loader
reachable_from:
  - task_runner.rs post-action hook (every ActionRecord ingested)
  - elai_cc.toml [tripwire] thresholds (operator-tunable, L3-locked from agent)
stub_ban:
  - kl_divergence MUST handle zero-baseline correctly (return INFINITY, not 0
    or NaN)
  - TripwireOrchestrator MUST NOT silently drop events when window full; trim
    oldest deterministically
  - Auto-revert MUST emit TripwireAutoRevert SSB BEFORE archive.retire (so
    audit trail captures intent even if revert fails)
  - Thresholds MUST load from elai_cc.toml, never hardcoded in TripwireOrchestrator
    constructor (defends against agent in-code edit per companion §4)
observable_effect:
  trigger: synthetic 100-action stream with 80% failure rate injected at
           action 60 (via test harness)
  expect_ssb: >= 3 TripwireFired rows from distinct kinds by action 65; >= 1
              TripwireFiredConcurrent; >= 1 TripwireAutoRevert; archive shows
              template retired with RetirementReason::Degradation
  expect_artifact: workflow/reasoning/D-4-decisions.md present; tripwire.toml
                   defaults match paper-cited values

## DETERMINISTIC_CHECK

```bash
# 1. SCHEMA_VERSION 50
rg "SCHEMA_VERSION: i64 = 50" .elai_cc/crates/orchestrator/src/ssb.rs

# 2. All 3 new SSB variants
rg "TripwireFired\b|TripwireFiredConcurrent|TripwireAutoRevert" \
  .elai_cc/crates/orchestrator/src/ssb.rs | wc -l
# Expected: >= 12

# 3. Tripwire types
rg "enum TripwireKind|struct TripwireConfig|struct DdmState|enum DdmLevel" \
  .elai_cc/crates/orchestrator/src/tripwire.rs

# 4. All 5 detector functions present
rg "fn kl_divergence|fn pearson_correlation|fn iqr_outlier_check|fn ppl_anomaly_check|fn update.*DdmState" \
  .elai_cc/crates/orchestrator/src/tripwire.rs

# 5. elai_cc.toml [tripwire] section + 4 published thresholds
rg "\[tripwire\]" .elai_cc/elai_cc.toml
rg "kl_spec_tau\s*=\s*0\.3" .elai_cc/elai_cc.toml
rg "rho_delta_threshold\s*=\s*0\.5" .elai_cc/elai_cc.toml
rg "iqr_outlier_multiplier\s*=\s*3\.0" .elai_cc/elai_cc.toml
rg "ppl_anomaly_sigma\s*=\s*2\.0" .elai_cc/elai_cc.toml

# 6. Build + scoped tests
cargo check --workspace 2>&1 | tail -3
cargo test -p elai_orchestrator -- tripwire 2>&1 | grep "test result"
cargo test -p elai_cc_contracts -- tripwire 2>&1 | grep "test result"

# 7. Paper-ingestor outputs
test -f workflow/papers/reward-hacking-detection-2507.05619-interface.md && \
  test -f workflow/papers/concept-drift-2004.05785-interface.md && \
  test -f workflow/papers/rm-overoptimization-2210.10760-interface.md && echo OK
```

## ACCEPTANCE

- [ ] SCHEMA_VERSION_PUBLIC == 50
- [ ] All 3 TripwireFired/Concurrent/AutoRevert variants + payloads round-trip
- [ ] All 5 detectors (KL, Δρ, IQR, perplexity, DDM) compile + unit tests pass
- [ ] DDM state machine transitions Normal → Warning → Drift on synthetic
      degradation; resets on recovery
- [ ] TripwireOrchestrator emits concurrent + auto-revert events when ≥2
      tripwires fire within window
- [ ] ArchiveChain.retire wired; revert marks template Retired with
      RetirementReason::Degradation; parent_template_id surfaced
- [ ] elai_cc.toml [tripwire] section present with paper-cited defaults; loader
      applies them
- [ ] Integration test (100-action injected-regression stream) detects ≥3
      tripwires by action 65; emits AutoRevert
- [ ] cargo check --workspace: zero new errors
- [ ] cargo test -p elai_orchestrator -- tripwire: all pass, ≥15 new tests
- [ ] cargo test -p elai_cc_contracts -- tripwire: all pass
- [ ] Cloud smoke test A: inject synthetic regression in test harness; observe
      ≥3 TripwireFired rows + AutoRevert + ArchiveChain.retire (verifiable
      via `elai archive verify` after D-8 lands; before D-8, verify via
      direct file inspection of reasoning-templates.jsonl)
- [ ] Cloud smoke test B: clean session of 50 cloud /task actions; verify
      ZERO TripwireFiredConcurrent rows (no false positives on healthy run)
- [ ] All 3 paper-ingestor outputs present
- [ ] Decision doc workflow/reasoning/D-4-decisions.md (3 decisions)
- [ ] memory-writer emits project_d4_tripwire_stack.md

## Files touched

- ADD: .elai_cc/crates/orchestrator/src/tripwire.rs
- MODIFY: .elai_cc/crates/orchestrator/src/ssb.rs (SCHEMA_VERSION 49→50;
    3 new SsbActionType variants + payloads)
- MODIFY: .elai_cc/crates/orchestrator/src/task_runner.rs (TripwireOrchestrator.
    ingest at post-action hook)
- MODIFY: .elai_cc/elai_cc.toml ([tripwire] section)
- MODIFY: .elai_cc/crates/orchestrator/src/lib.rs (pub mod tripwire)
- ADD: .elai_cc/crates/orchestrator/tests/tripwire_integration.rs
- ADD: .elai_cc/crates/contracts/tests/tripwire_contract.rs
- ADD: workflow/reasoning/D-4-decisions.md
- ADD: workflow/papers/{reward-hacking-detection-2507.05619,concept-drift-
       2004.05785,rm-overoptimization-2210.10760}-interface.md

## Out-of-scope

- D-5 BAI Thompson (consumes tripwire fires as posterior input but defines
  its own algorithm surface)
- D-8 Archive verify CLI (surfaces revert/retire chain to user; auto-revert
  uses ArchiveChain.retire internal API here)
- D-9 LockedKeysAuditor (enforces tripwire-config L3 lock; here we only
  document the L3 status)
- D-26 dashboard surface (renders tripwire stream)

## Next

D-5 (BAI Thompson promotion) consumes VerifierConsensusPass count + tripwire
fires as arm-posterior inputs. D-6 (held-out store) provides held-out signal
that is one input to KL + Δρ detectors (held-out distribution as baseline).
