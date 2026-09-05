# D-3: VerificationResult aggregator + multi-verifier consensus rule

Objective: aggregate deterministic-check + PPM score + intrinsic GG signals
+ cross-context agreement into a typed VerificationResult; require ≥2 of 4
verifier signals to concur on improvement before any template promotion. This
is the V-STaR pattern (arxiv:2402.06457, T1) extended to ELAI's multi-mode
substrate. PRMBench (arxiv:2501.03124, T1) Deception Resistance + Circular
Logic categories — exactly what a self-improving system gaming the verifier
would exploit — are defended structurally by the consensus rule, not just
prompt-level disclaimers.
Estimated effort: 1-2 sessions.
Pre-requisites: D-2 (PpmClient + PpmAdapter exist; PPM is one of the 4
  signals), D-1 (ReasoningTemplate.verifier_required field consumed; selects
  applicable signals per template).
Parallel-safe with: D-5 (BAI Thompson promotion; consumes VerificationResult
  output but defined on its own algorithmic surface). NOT parallel-safe with
  D-4 (Tripwire stack monitors VerifierConsensusReject signals).

## Context

Companion §3.3 (lines 242-256) defines the consensus aggregator. V-STaR
(arxiv:2402.06457, T1) demonstrates 4-17% test accuracy gain when the verifier
is trained on BOTH correct AND incorrect generations and ACTS as a separate
artifact; combined with rStar-Math (arxiv:2501.04519, T1) the policy and
verifier evolve TOGETHER but SEPARATELY, against DIFFERENT signals — single-
verifier sign-off is the failure mode.

PRMBench (arxiv:2501.03124, T1) §4.2 reports that 25 evaluated PRMs and
LLM-judges score worst on Deception Resistance and Circular Logic — exactly
the categories a gaming self-improver would target. Single-PPM sign-off is
unsafe; consensus across signals from different sources (deterministic /
PPM / intrinsic / cross-context) is the structural defense.

GG-signals already partially defined in ELAI (Phase 4G groundedness check)
but not currently composed into a VerificationResult: mean_logprob, entropy_max,
novelty (rolling Jaccard against recent outputs). These exist as scattered
metrics in task_runner.rs; D-3 collects them into IntrinsicSignals.

Cross-context agreement is from arxiv:2603.21454 (T1): same claim run in
two independent context windows; agreement = robust, disagreement = flag.
ELAI does not currently implement this; D-3 wires the dispatch (executor runs
twice with different prior-step ordering, agreement computed).

Partial foundation present and usable:
- DeterministicCheck via SlashCommand::Dispatch (compile/test/lint hooks).
- PpmClient (D-2) + PpmAdapter.
- GG-signal field on AgentState (Phase 4G).
- ReasoningTemplate.verifier_required (D-1) drives which signals are required
  per template.

## Reads

- F:\.ELAI_workflow\docs\specs\guide\reasoning-improvement-grounded-2026-05-19.md
  (§3.3 VerificationResult; §1.7 V-STaR; §1.13 PRMBench weaknesses —
  lines 70-74 + 117-126 + 242-256)
- F:\.ELAI_workflow\workflow\papers\latent-verifiers-2504.16760-interface.md
  (D-2 ingested; heterogeneous-verifier rule)
- F:\.ELAI_workflow\workflow\papers\prmbench-2501.03124-interface.md
  (D-2 ingested; Deception/Circular categories)
- F:\.ELAI_workflow\workflow\papers\rstar-math-2501.04519-interface.md
  (D-2 ingested; verifier-evolves-separately)
- .elai_cc/crates/orchestrator/src/ppm_client.rs (D-2 PpmClient)
- .elai_cc/crates/orchestrator/src/reasoning_template.rs (D-1 VerifierKind)
- .elai_cc/crates/reasoning/src/engine.rs (existing GRC scoring path; GG-signal
  emission site)
- .elai_cc/crates/orchestrator/src/task_runner.rs (current single-verifier paths
  at deterministic-check dispatch — emit sites for VerificationResult)

## Paper-ingestor pre-step

Spawn paper-ingestor with mode=paper for:
- arxiv:2402.06457 V-STaR (consensus / separable-verifier substrate)
- arxiv:2603.21454 Cross-Context Verification (CCV signal)

Verify workflow/papers/{vstar-2402.06457,ccv-2603.21454}-interface.md before
unblocking TESTING_FIRST.

## Tasks

1. **VerificationResult struct** in new file
   .elai_cc/crates/orchestrator/src/verification_result.rs:
   - `struct VerificationResult { deterministic: Option<DeterministicCheck>,
     ppm: Option<PpmScore>, intrinsic: Option<IntrinsicSignals>, cross_context:
     Option<CrossContextAgreement>, consensus_rule: ConsensusRule, verdict:
     Verdict }`
   - `struct DeterministicCheck { kind: DeterministicKind, passed: bool,
     stderr_snippet: Option<String> }`; `enum DeterministicKind { Compile,
     Test, Lint, Schema, RegistryCheck }`
   - `struct IntrinsicSignals { mean_logprob: f32, entropy_max: f32,
     novelty: f32 }` (each in well-defined range; documented in struct)
   - `struct CrossContextAgreement { context_a_hash: Blake3Hash, context_b_hash:
     Blake3Hash, jaccard_similarity: f32, agreement: bool }` (agreement =
     similarity >= 0.7 threshold; threshold value in elai_cc.toml [verification]
     section, L3-locked)
   - `enum ConsensusRule { AtLeastTwo, AtLeastThree, AtLeastN(u8),
     RequireAll(Vec<SignalKind>) }` (closed enum; L3-locked per companion §4)
   - `enum SignalKind { Deterministic, Ppm, Intrinsic, CrossContext }`
   - `enum Verdict { Pass, Reject, Inconclusive }`

2. **Consensus computation** in verification_result.rs:
   - `impl VerificationResult`: compute_verdict(rule: ConsensusRule) -> Verdict.
   - AtLeastTwo: count signals where individual verdict = Pass (DeterministicCheck.
     passed, PpmScore.score >= 0.7, IntrinsicSignals.mean_logprob >=
     intrinsic_threshold, CrossContextAgreement.agreement). >= 2 -> Pass;
     0 or 1 -> Reject; only Inconclusive if a required signal is None.
   - AtLeastN(n): same, threshold n.
   - RequireAll(kinds): every kind in `kinds` must be present AND Pass.
   - Default rule: AtLeastTwo (per companion §3.3 spec).

3. **CrossContextRunner** in
   .elai_cc/crates/orchestrator/src/cross_context.rs:
   - `struct CrossContextRunner { executor: Arc<dyn Executor>, jaccard_threshold:
     f32 }`
   - `fn run(&self, claim: &Claim, ctx: &OrchContext) -> CrossContextAgreement`:
     spawns 2 parallel executor calls with prior_steps shuffled; computes
     blake3 hash of each output; jaccard similarity on token-level diff
     (use shingles n=3 per arxiv:2603.21454 §3.2); agreement = similarity >=
     threshold.
   - Bounded latency: 30s max per side (timeout returns Inconclusive, not
     panic).
   - Per CLAUDE.md no-blocking rule: cross-context is async; orchestrator awaits
     both side futures concurrently.

4. **Verifier dispatch in task_runner.rs**:
   - `fn dispatch_verifiers(template: &ReasoningTemplate, candidate: &Candidate,
     ctx: &OrchContext) -> VerificationResult`:
     - Always dispatch deterministic check (compile / test if applicable; skip
       Inapplicable per checker registry from D-11 — but D-11 not landed yet,
       so use existing deterministic hooks).
     - If template.verifier_required == VerifierKind::Prm AND ELAI_PPM_ENABLE
       AND PpmClient.health() OK: dispatch PPM.
     - Always emit IntrinsicSignals from current GG-signal block.
     - If consensus_rule requires SignalKind::CrossContext: dispatch
       CrossContextRunner.
     - Compose VerificationResult; compute_verdict; emit SSB row.

5. **SSB action types + SCHEMA_VERSION bump (atomic with D-2 scaffolds)**
   in ssb.rs:
   - Bump SCHEMA_VERSION 48→49 (atomic for D-2 PpmScoreEmitted/PpmUnavailable
     + D-3 verifier types).
   - New variants:
     - `VerifierConsensusPass { template_id: TemplateId, candidate_id: Uuid,
       signals: Vec<SignalKind>, consensus_rule: String }`
     - `VerifierConsensusReject { template_id: TemplateId, candidate_id: Uuid,
       failing_signals: Vec<(SignalKind, String)>, consensus_rule: String }`
     - `VerifierDisagreement { template_id: TemplateId, signals: Vec<(SignalKind,
       String)> }` (operational telemetry; fires when ≥1 Pass + ≥1 Reject)
     - `CrossContextDispatched { claim_id: Uuid, context_a_hash: String,
       context_b_hash: String, jaccard: f64, agreement: bool }`
   - All variants: SsbPayload arm + SQL serialization + round-trip test.

6. **elai_cc.toml [verification] section**:
   - `[verification]\nconsensus_rule = "at_least_two"\nintrinsic_logprob_threshold
     = -3.0\nintrinsic_entropy_max = 3.5\nintrinsic_novelty_min = 0.2\n
     cross_context_jaccard_threshold = 0.7`
   - L3-LOCKED: these values are evaluator-side knobs; agent must not edit
     (D-9 enforces).

7. **Unit tests** in verification_result.rs + cross_context.rs:
   - AtLeastTwo with 2 passing signals -> Pass; with 1 -> Reject; with 0 -> Reject.
   - RequireAll([Deterministic, Ppm]) with both present + passing -> Pass;
     with Ppm=None -> Inconclusive.
   - Consensus rule deserialize from elai_cc.toml string.
   - CrossContextRunner returns Inconclusive on either-side timeout; doesnt
     panic.
   - Jaccard similarity helper deterministic on identical inputs.
   - DisagreementSignal detection: 1 Pass + 1 Reject + others None -> emit
     VerifierDisagreement row.

8. **Integration test** in
   .elai_cc/crates/orchestrator/tests/verifier_consensus_integration.rs:
   - Mock executor + mock PPM + mock deterministic check.
   - Scenario 1: deterministic Pass + PPM Pass + intrinsic Pass = consensus Pass.
   - Scenario 2 (PRMBench-style deceptive): deterministic Fail + PPM Pass +
     intrinsic Pass = consensus Reject (deterministic Pass-or-Fail is treated
     as a binary signal contributing 0 or 1 to the count; the failure breaks
     the AtLeastTwo threshold).
   - Wait — actually re-reading: deterministic Fail is its own signal kind
     where "Pass" of the signal == passed=true. So in Scenario 2: signals
     contributing Pass = {Ppm, Intrinsic} = 2 -> consensus Pass. That defeats
     the example. The correct PRMBench-deceptive scenario should require
     ConsensusRule::RequireAll([Deterministic, Ppm]) for code claims.
   - Update Scenario 2: code claim with RequireAll([Deterministic, Ppm]);
     Deterministic Fail -> consensus Reject. This matches the spec better.
   - Scenario 3: cross-context disagreement -> Inconclusive (no consensus).
   - Per-test SSB rows asserted via in-memory SSB writer.

9. **Contract test** crates/contracts/tests/verifier_consensus_contract.rs:
   - SCHEMA_VERSION_PUBLIC == 49.
   - All new SSB variants + payloads round-trip.
   - ConsensusRule serde stable across edits.
   - Default rule (AtLeastTwo) applied when elai_cc.toml [verification] absent.

10. **Decision doc** workflow/reasoning/D-3-decisions.md (3 decisions):
    - (a) Why AtLeastTwo default and not AtLeastThree: companion §3.3 +
      practical observation that requiring all 4 signals always-on creates
      gate explosion + PPM hot-path latency. Default tunable per template_kind
      in D-9 L3 config.
    - (b) Why blake3 hash on cross-context outputs (not edit-distance): blake3
      is content-hash for deduplication; jaccard on shingles is the similarity
      metric (different from the hash).
    - (c) Why VerifierDisagreement is a separate SSB row (not subsumed into
      VerifierConsensusReject): tripwire stack (D-4) monitors disagreement
      rate as an early signal of verifier drift; rejecting and disagreeing
      are operationally distinct.

## WIRING_CONTRACT

produces_symbols:
  - VerificationResult + Verdict
  - DeterministicCheck + DeterministicKind
  - IntrinsicSignals
  - CrossContextAgreement + CrossContextRunner
  - ConsensusRule (closed enum) + SignalKind (closed enum)
  - compute_verdict logic
  - dispatch_verifiers (task_runner integration)
  - SsbActionType::VerifierConsensusPass + ::VerifierConsensusReject +
    ::VerifierDisagreement + ::CrossContextDispatched + ::PpmScoreEmitted
    (atomic with D-2) + ::PpmUnavailable + payloads
  - SCHEMA_VERSION bump 48→49 (atomic with D-2 scaffold)
  - [verification] elai_cc.toml section
consumes_symbols:
  - PpmClient (D-2)
  - ReasoningTemplate.verifier_required (D-1)
  - GG-signal block in AgentState (Phase 4G)
  - SsbWriter (orchestrator/src/ssb.rs)
  - SeededClock (Phase 4G; for cross-context timeout deadline)
reachable_from:
  - task_runner.rs dispatch_verifiers call (every CLAIM-emitting baby step
    after EXECUTING)
  - D-5 BAI promotion consumes VerifierConsensusPass count
  - D-4 tripwire monitors VerifierDisagreement rate
stub_ban:
  - compute_verdict MUST NOT swallow None signals; explicit Inconclusive verdict
  - CrossContextRunner MUST timeout per side, no infinite wait
  - dispatch_verifiers MUST emit one VerificationResult-shaped SSB row per
    candidate (no silent skip when a signal is unavailable)
  - ConsensusRule MUST be loaded from elai_cc.toml; no in-code constant
    (defends against agent in-code edit; rule lives in L3 config)
observable_effect:
  trigger: `elai /task --mode cloud "synthetic deceptive reasoning task"` where
           the candidate body passes PPM but fails compile
  expect_ssb: 1 VerifierConsensusReject row with failing_signals containing
              Deterministic + an associated stderr snippet; PpmScoreEmitted row
              showing PPM scored it as Pass (deception caught by deterministic)
  expect_artifact: candidate is NOT promoted (no TemplatePromoted row)

## DETERMINISTIC_CHECK

```bash
# 1. SCHEMA_VERSION 49 (atomic bump for D-2 + D-3)
rg "SCHEMA_VERSION: i64 = 49" .elai_cc/crates/orchestrator/src/ssb.rs

# 2. New SSB variants present (count all 6 from D-2 + D-3 atomic)
rg "PpmScoreEmitted|PpmUnavailable|VerifierConsensusPass|VerifierConsensusReject|VerifierDisagreement|CrossContextDispatched" \
  .elai_cc/crates/orchestrator/src/ssb.rs | wc -l
# Expected: >= 24

# 3. VerificationResult + ConsensusRule
rg "struct VerificationResult|enum ConsensusRule|enum SignalKind|enum Verdict" \
  .elai_cc/crates/orchestrator/src/verification_result.rs

# 4. CrossContextRunner
rg "struct CrossContextRunner|fn run" .elai_cc/crates/orchestrator/src/cross_context.rs

# 5. elai_cc.toml [verification] section
rg "\[verification\]" .elai_cc/elai_cc.toml

# 6. dispatch_verifiers integration in task_runner
rg "dispatch_verifiers" .elai_cc/crates/orchestrator/src/task_runner.rs

# 7. Build + scoped tests
cargo check --workspace 2>&1 | tail -3
cargo test -p elai_orchestrator -- verification_result cross_context 2>&1 | grep "test result"
cargo test -p elai_cc_contracts -- verifier_consensus 2>&1 | grep "test result"

# 8. No single-verifier sign-off path
rg "single.verifier\|verifier.only.*sign\|skip.consensus" \
  .elai_cc/crates/orchestrator/src/

# 9. Paper-ingestor outputs
test -f workflow/papers/vstar-2402.06457-interface.md && \
  test -f workflow/papers/ccv-2603.21454-interface.md && echo OK
```

## ACCEPTANCE

- [ ] SCHEMA_VERSION_PUBLIC == 49 (atomic bump D-2 + D-3); old DB version 48
      emits migration warn
- [ ] All 6 SSB variants (PpmScoreEmitted, PpmUnavailable, VerifierConsensusPass,
      VerifierConsensusReject, VerifierDisagreement, CrossContextDispatched)
      present + round-trip pass
- [ ] VerificationResult + ConsensusRule + Verdict compile; unit tests pass
- [ ] AtLeastTwo / AtLeastN / RequireAll consensus computation correct under
      table-driven tests (≥10 scenarios)
- [ ] CrossContextRunner returns Inconclusive on timeout (does not panic)
- [ ] dispatch_verifiers integrated at task_runner.rs CLAIM-emit sites; emits
      one VerificationResult per candidate
- [ ] elai_cc.toml [verification] section loaded; defaults applied when absent
- [ ] cargo check --workspace: zero new errors
- [ ] cargo test -p elai_orchestrator -- verification_result cross_context: all
      pass, ≥12 new tests
- [ ] cargo test -p elai_cc_contracts -- verifier_consensus: all pass
- [ ] Cloud smoke test A: synthetic candidate that passes PPM but fails compile
      -> VerifierConsensusReject SSB row; not promoted
- [ ] Cloud smoke test B: cross-context agreement >= 0.7 on simple math claim;
      CrossContextDispatched SSB row with agreement=true
- [ ] Cloud smoke test C: PPM offline + ELAI_PPM_ENABLE=1 + template requires
      PPM -> Inconclusive verdict; PpmUnavailable + VerifierConsensusReject
      (or Inconclusive-flag) row depending on consensus rule
- [ ] Paper-ingestor outputs (vstar, ccv) present
- [ ] Decision doc workflow/reasoning/D-3-decisions.md (3 decisions)
- [ ] memory-writer emits project_d3_verification_result.md

## Files touched

- ADD: .elai_cc/crates/orchestrator/src/verification_result.rs
- ADD: .elai_cc/crates/orchestrator/src/cross_context.rs
- MODIFY: .elai_cc/crates/orchestrator/src/ssb.rs (SCHEMA_VERSION 48→49 atomic
    bump; 6 new variants from D-2 + D-3)
- MODIFY: .elai_cc/crates/orchestrator/src/task_runner.rs (dispatch_verifiers
    at CLAIM-emit sites)
- MODIFY: .elai_cc/elai_cc.toml ([verification] section)
- MODIFY: .elai_cc/crates/orchestrator/src/lib.rs (pub mod verification_result;
    pub mod cross_context)
- ADD: .elai_cc/crates/orchestrator/tests/verifier_consensus_integration.rs
- ADD: .elai_cc/crates/contracts/tests/verifier_consensus_contract.rs
- ADD: workflow/reasoning/D-3-decisions.md
- ADD: workflow/papers/{vstar-2402.06457,ccv-2603.21454}-interface.md

## Out-of-scope

- D-4 Tripwire stack (consumes VerifierDisagreement signal)
- D-5 BAI Thompson (consumes VerifierConsensusPass for arm posterior update)
- D-11 Checker registry (replaces ad-hoc deterministic hooks with typed checkers)
- D-13 Adversary role (consumes consensus result + injects cross-family signal)

## Next

D-4 (tripwire stack) and D-5 (BAI Thompson) can run in parallel; both consume
VerificationResult output but operate on disjoint algorithmic surfaces.
