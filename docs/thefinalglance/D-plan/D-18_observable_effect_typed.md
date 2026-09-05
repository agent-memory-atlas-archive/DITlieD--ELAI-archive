# D-18: ObservableEffect typed predicate (Mode D upgrade, Phase 4F extension)

Objective: extend C-22 FeasibilityProber with typed ObservableEffect struct;
require ≥2 ObservableEffects per claim (orthogonal aspects); distinguish
SandboxObservation from ProductionObservation. Realizes Voyager (arxiv:
2305.16291, T1) skill-with-effect + Eureka (arxiv:2310.12931, T1) reward-as-
falsifier + Coscientist (arxiv:2304.05332, T1) safety-failure-mode docs as
load-bearing primitives (real-problem §4.4 lines 253-273).
Estimated effort: 2 sessions.
Pre-requisites: C-22 (FeasibilityProber completeness landed), D-11 (Mode D
  uses checkers as effect verifiers).
Parallel-safe with: D-17 SLA monitoring.

## Context

Real-problem §4.4 (lines 253-273). C-22 already adds FeasibilityProber
completeness + A1-A8 adversarial tests + FeasibilityPanel.svelte. D-18 adds
the typed ObservableEffect schema + multi-effect requirement + sandbox/
production observation labeling.

Failure modes acknowledged in real-problem §4.4:
- **Goodhart on test design**: require ≥2 ObservableEffects per claim, each
  measuring orthogonal aspect.
- **Simulator-reality gap**: SandboxObservation ≠ ProductionObservation;
  NEVER claim production grounding from sandbox alone.
- **Confounded experiment**: log all environmental variables; confound_flags
  field surfaces known risks.

Partial foundation present and usable:
- C-22 FeasibilityProber + DefaultFeasibilityProber + A1-A8 tests.
- D-11 CheckerRegistry — checkers are the effect runners.
- B-18 + C-11 Tier C sandbox.

## Reads

- F:\.ELAI_workflow\docs\specs\guide\real-problem-solving-grounded-2026-05-19.md
  (§4.4 ObservableEffect — lines 253-273)
- F:\.ELAI_workflow\docs\thefinalglance\C-plan\C-22_feasibility_probe_completeness.md
- .elai_cc/crates/orchestrator/src/checker_registry.rs (D-11)
- .elai_cc/crates/feasibility/ or wherever FeasibilityProber lives (C-22)

## Paper-ingestor pre-step

Spawn paper-ingestor with mode=paper for:
- arxiv:2305.16291 Voyager (skill-effect pattern)
- arxiv:2310.12931 Eureka (reward design / sim-real falsification)
- arxiv:2304.05332 Coscientist (real-world experiment grounding + confounds)

Verify workflow/papers/{voyager-2305.16291,eureka-2310.12931,coscientist-
2304.05332}-interface.md.

## Tasks

1. **ObservableEffect types** in new file
   .elai_cc/crates/orchestrator/src/observable_effect.rs:
   - `struct ObservableEffect { trigger: TriggerSpec, expected_observation:
     TypedObservation, success_predicate: Predicate, bounded_runtime:
     Duration, sandbox_tier: SandboxTier, confound_flags: Vec<ConfoundFlag> }`
   - `struct TriggerSpec { kind: TriggerKind, payload: BoundedString<4096> }`
   - `enum TriggerKind { Subprocess, HttpRequest, FileWrite, SandboxedRun }`
   - `enum TypedObservation { StringMatch(BoundedString<2048>),
     FileContent { path: BoundedString<512>, expected_blake3:
     Option<Blake3Hash> }, ExitCode(i32), MetricValue { metric_name: String,
     range: (f64, f64) } }`
   - `enum SandboxTier { TierA, TierB, TierC }` (matches B-18 / C-11)
   - `enum ConfoundFlag { ColdCache, ColdNetwork, GpuRecentlyIdle,
     ConcurrentTasks, Other(String) }`

2. **MultiEffect requirement** in observable_effect.rs:
   - `struct EffectSet { effects: Vec<ObservableEffect>, claim_id: ClaimId,
     target_orthogonality: u8 }` (default ≥2)
   - `fn validate_orthogonality(set: &EffectSet) -> Result<(),
     OrthogonalityError>`:
     - Require effects.len() >= target_orthogonality.
     - Heuristic orthogonality: effects MUST differ on at least one of
       (trigger.kind, expected_observation kind variant). Same-kind effects
       count as one for orthogonality.

3. **Observation labeling** in observable_effect.rs:
   - `enum ObservationContext { Sandbox(SandboxTier), Production }`
   - `struct ObservationResult { effect: ObservableEffect, context:
     ObservationContext, outcome: ObservationOutcome, timing_ms: u64,
     environment_snapshot: EnvironmentSnapshot }`
   - `enum ObservationOutcome { Confirmed, Refuted { evidence:
     BoundedString<2048> }, Inconclusive { reason: String } }`
   - `struct EnvironmentSnapshot { hostname: String, hardware_class: String,
     active_confounds: Vec<ConfoundFlag> }`
   - ALL sandbox observations are labeled with Sandbox(tier); promotion from
     sandbox to production-claim REQUIRES explicit operator ratification (no
     auto-promotion; documented in real-problem §4.4 simulator-reality-gap
     constraint).

4. **FeasibilityProber integration** in feasibility/lib.rs (C-22 extension):
   - DefaultFeasibilityProber.probe(...) now returns Vec<ObservationResult>
     instead of single result.
   - Probe construction MUST include EffectSet with ≥2 effects;
     validate_orthogonality enforced at probe-author time.

5. **Goodhart-risk flag** in observable_effect.rs:
   - `fn check_goodhart_risk(set: &EffectSet) -> Option<GoodhartRiskFlag>`:
     If effects.len() < target_orthogonality → emit flag with reason
     "single-effect probe vulnerable to Goodhart".

6. **SSB action types** in ssb.rs (SCHEMA_VERSION 63→64):
   - `ObservableEffectDispatched { claim_id: ClaimId, effect_count: u32,
     orthogonality_target: u8, context: String }`
   - `GoodhartRiskFlag { claim_id: ClaimId, effect_count: u32, reason: String }`
   - `SandboxObservation { claim_id: ClaimId, effect_id: u32, outcome: String,
     tier: String }`
   - `ProductionObservationProposal { claim_id: ClaimId, sandbox_evidence:
     String, operator_required: bool }`
   - Round-trip tests.

7. **elai_cc.toml [observable_effect] section**:
   - `[observable_effect]\nmin_effects_per_claim = 2\ntarget_orthogonality = 2\n
     auto_promote_sandbox_to_production = false`
   - L3-LOCKED (especially auto_promote_sandbox_to_production = false).

8. **Unit tests** in observable_effect.rs:
   - validate_orthogonality on 1 effect → OrthogonalityError.
   - validate_orthogonality on 2 same-kind effects → OrthogonalityError.
   - validate_orthogonality on 2 different-kind effects → Ok.
   - check_goodhart_risk on insufficient effects → Some flag.
   - ObservationContext::Sandbox labeling preserved through ObservationResult.

9. **Integration test** in
   .elai_cc/crates/orchestrator/tests/observable_effect_integration.rs:
   - Probe with 1 effect → GoodhartRiskFlag SSB row.
   - Probe with 2 orthogonal effects → ObservableEffectDispatched + 2
     SandboxObservation rows.
   - Synthetic probe that succeeds in sandbox; verify NO production claim
     auto-emitted (auto_promote=false).

10. **Decision doc** workflow/reasoning/D-18-decisions.md (3 decisions):
    - (a) Why ≥2 orthogonal effects: real-problem §4.4 — single-effect
      probes vulnerable to Goodhart on test design; orthogonal coverage
      defends.
    - (b) Why sandbox/production labeling explicit: real-problem §4.4 —
      Voyager works in Minecraft, not field; Coscientist documents confounds.
      Explicit labeling prevents premature promotion claims.
    - (c) Why no auto-promotion: operator-in-loop is the safety check;
      auto-promotion would let optimizer climb proxy reward.

## WIRING_CONTRACT

produces_symbols:
  - ObservableEffect + TriggerSpec + TriggerKind + TypedObservation +
    SandboxTier + ConfoundFlag (closed enums)
  - EffectSet + validate_orthogonality + check_goodhart_risk
  - ObservationContext + ObservationResult + ObservationOutcome +
    EnvironmentSnapshot (closed enums)
  - SsbActionType::ObservableEffectDispatched + ::GoodhartRiskFlag +
    ::SandboxObservation + ::ProductionObservationProposal + payloads
  - SCHEMA_VERSION bump 63→64
  - [observable_effect] elai_cc.toml section
consumes_symbols:
  - FeasibilityProber (C-22)
  - CheckerRegistry (D-11; checkers run individual effects)
  - Tier C sandbox (B-18 + C-11)
  - Mode D dispatch (D-10)
reachable_from:
  - Mode D Operational Falsification branch (D-10)
  - FeasibilityProber.probe entry point
stub_ban:
  - validate_orthogonality MUST require ≥ target_orthogonality
  - auto_promote_sandbox_to_production MUST default to false (operator gate)
  - check_goodhart_risk MUST emit flag on single-effect probes
  - ObservationContext label MUST persist through ObservationResult
observable_effect:
  trigger: feasibility probe of a claim with 2 orthogonal ObservableEffects
  expect_ssb: 1 ObservableEffectDispatched row; 2 SandboxObservation rows;
              0 GoodhartRiskFlag; 0 ProductionObservationProposal (until
              operator promotes)
  expect_artifact: workflow/sessions/{id}/observations/{claim_id}.toml
                   persisted with environment_snapshot + active_confounds

## DETERMINISTIC_CHECK

```bash
rg "SCHEMA_VERSION: i64 = 64" .elai_cc/crates/orchestrator/src/ssb.rs
rg "ObservableEffectDispatched|GoodhartRiskFlag|SandboxObservation|ProductionObservationProposal" \
  .elai_cc/crates/orchestrator/src/ssb.rs | wc -l
# Expected: >= 16
rg "struct ObservableEffect|struct EffectSet|fn validate_orthogonality|fn check_goodhart_risk" \
  .elai_cc/crates/orchestrator/src/observable_effect.rs
rg "\[observable_effect\]" .elai_cc/elai_cc.toml
rg "auto_promote_sandbox_to_production\s*=\s*false" .elai_cc/elai_cc.toml
cargo check --workspace 2>&1 | tail -3
cargo test -p elai_orchestrator -- observable_effect 2>&1 | grep "test result"
test -f workflow/papers/voyager-2305.16291-interface.md && \
  test -f workflow/papers/eureka-2310.12931-interface.md && \
  test -f workflow/papers/coscientist-2304.05332-interface.md && echo OK
```

## ACCEPTANCE

- [ ] SCHEMA_VERSION_PUBLIC == 64
- [ ] 4 SSB variants round-trip
- [ ] ObservableEffect + EffectSet + validate_orthogonality compile
- [ ] Orthogonality enforcement: 1 effect → error; 2 same-kind → error;
      2 orthogonal → ok
- [ ] Sandbox/production labeling preserved through ObservationResult
- [ ] check_goodhart_risk emits flag on insufficient effects
- [ ] auto_promote_sandbox_to_production = false enforced
- [ ] FeasibilityProber integration (C-22) returns Vec<ObservationResult>
- [ ] elai_cc.toml [observable_effect] loaded
- [ ] cargo check --workspace: zero new errors
- [ ] cargo test -p elai_orchestrator -- observable_effect: all pass, ≥10
      tests
- [ ] Cloud smoke test A: feasibility probe with 1 effect → GoodhartRiskFlag
      SSB row
- [ ] Cloud smoke test B: feasibility probe with 2 orthogonal effects →
      ObservableEffectDispatched + 2 SandboxObservation; sandbox/production
      distinction preserved
- [ ] Cloud smoke test C: sandbox-passing probe → NO automatic
      production-claim emission (auto_promote=false enforced)
- [ ] All 3 paper-ingestor outputs present
- [ ] Decision doc workflow/reasoning/D-18-decisions.md

## Files touched

- ADD: .elai_cc/crates/orchestrator/src/observable_effect.rs
- MODIFY: .elai_cc/crates/feasibility/src/lib.rs (C-22 prober return-type
    extension)
- MODIFY: .elai_cc/crates/orchestrator/src/ssb.rs (SCHEMA_VERSION 63→64;
    4 new variants)
- MODIFY: .elai_cc/crates/orchestrator/src/grounding_mode.rs (Mode D dispatch
    invokes ObservableEffect runner)
- MODIFY: .elai_cc/elai_cc.toml ([observable_effect] section)
- MODIFY: .elai_cc/crates/orchestrator/src/lib.rs (pub mod observable_effect)
- ADD: .elai_cc/crates/orchestrator/tests/observable_effect_integration.rs
- ADD: workflow/reasoning/D-18-decisions.md
- ADD: workflow/papers/{voyager-2305.16291,eureka-2310.12931,coscientist-
       2304.05332}-interface.md

## Out-of-scope

- Production-claim CLI for operator ratification (DEFERRED; sandbox-only
  for now)
- Per-domain effect template library (DEFERRED; ad-hoc authoring first)
- Automated confound detection (DEFERRED; manual flagging only)

## Next

Wave 2 complete. Wave 3 (D-19..D-25) is benchmarks + adversarial tests that
gate the Wave-1/Wave-2 capabilities.
