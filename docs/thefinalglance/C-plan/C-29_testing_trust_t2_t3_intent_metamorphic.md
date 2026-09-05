# C-29: Testing-Trust T-2 (IntentPair) + T-3 basic Metamorphic Relations

Objective: implement T-2 IntentPair formalization layer and the T-3 basic
metamorphic relations subset (MR_identity, MR_inverse, MR_permutation_invariance)
per testing-trust-design.txt. Add INTENT_CONFIRMED and MR_VIOLATED SSB action
types (SCHEMA_VERSION 47). Block PLANNING -> TESTING_FIRST gate requiring >= 3
IntentPairs per BabyStepSpec. Wire MetamorphicTestRunner into auditor pipeline
after Mutation tier. Add IntentPair editor dialog to PlanWindow.svelte.
Estimated effort: 2-3 sessions.
Pre-requisites: C-27 (T-0 WiringContract layer), C-28 (T-1 mutation tier wired).
Parallel-safe with: C-21, C-22, C-23, C-24, C-25, C-26.

## Context

testing-trust-design.report.md §Mechanisms MISSING (report:68-77) confirms
IntentPair, IntentPairSet, IntentConfirmation types absent; INTENT_CONFIRMED SSB
action type absent; PLANNING -> TESTING_FIRST gate requiring >= 3 pairs absent.
MetamorphicRelation, RelationLibrary absent (report:74-77). No MR_VIOLATED SSB
type. No MetamorphicTestRunner in auditor pipeline.

Foundation available: BabyStepSpec at research_tree.rs:1667 has tdd, acceptance,
preferred_mode fields — intent_pairs field adds cleanly with #[serde(default)].
GateDecision enum at gate.rs:25 with existing PLANNING->TESTING_FIRST path at
gate.rs:583. SsbActionType at ssb.rs currently SCHEMA_VERSION 46 (B-37C). TddSpec
at research_tree.rs:1731 establishes the pattern for per-step formalization types.
PlanWindow.svelte at crates/dashboard-v2/src/lib/windows/PlanWindow.svelte has
existing BabyStepSpec rendering surface. proptest infrastructure active in codebase
(research_tree.rs, knowledge/proptest-regressions/).

## Reads

- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\guide\testing-trust-design.report.md
- .elai_cc/crates/orchestrator/src/research_tree.rs:1667-1760 (BabyStepSpec, TddSpec)
- .elai_cc/crates/orchestrator/src/gate.rs:25,133,583 (GateDecision, PLANNING->TESTING_FIRST path)
- .elai_cc/crates/orchestrator/src/ssb.rs:173-189 (SCHEMA_VERSION constant)
- .elai_cc/crates/orchestrator/src/autonomous_gate.rs (autonomous gate dispatch pattern)
- .elai_cc/crates/orchestrator/src/deterministic_auditor.rs (auditor pipeline insertion point)
- .elai_cc/crates/dashboard-v2/src/lib/windows/PlanWindow.svelte (BabyStep rendering)
- elai_cc.toml (confirm [testing] section absent; locate [testing.intent] insertion point)

## Tasks

1. **IntentPair + IntentPairSet + IntentConfirmation types** in new file
   .elai_cc/crates/orchestrator/src/intent_pair.rs.
   IntentPair { natural_language_intent: String, concrete_test_assertion: String,
   baby_step_ref: BabyStepId }. IntentPairSet { pairs: Vec<IntentPair>,
   confirmed: bool, confirmed_at: Option<DateTime<Utc>> }. Methods: len();
   is_confirmed(); confirm() sets confirmed=true + timestamp.
   IntentConfirmation { step_id: BabyStepId, pair_count: usize,
   confirmed_by: ConfirmedBy, timestamp: DateTime<Utc> }.
   ConfirmedBy enum { User, AutonomousGate }. Derive Debug/Clone/Serialize/
   Deserialize on all. No .unwrap(); use ? propagation.

2. **Extend BabyStepSpec** in research_tree.rs: add
   `#[serde(default)] pub intent_pairs: IntentPairSet` after target_file.
   Default produces empty unconfirmed set; existing JSON deserializes without
   schema bump.

3. **SSB action types** (SCHEMA_VERSION 47). In ssb.rs add comment:
   `// C-29 (SCHEMA_VERSION 47): T-2 intent confirmation + T-3 MR violation.`
   Bump constant to 47. Add SsbActionType::IntentConfirmed { step_id: u64,
   pair_count: u8, confirmed_by: String } and SsbActionType::MrViolated
   { step_id: u64, relation: String, counterexample: String, sample_size: u32 }.
   Add payload structs; wire SsbPayload match arms per B-37C pattern at ssb.rs:1126.

4. **Config: [testing.intent]**. Add to elai_cc.toml (grep first to locate [testing]
   parent; Anti-Slopsquatting). `[testing.intent] min_pairs = 3`. Add
   IntentConfig { min_pairs: usize } in shared config module (grep elai.map for
   "struct.*Config" in orchestrator to locate). Wire into gate check (Task 5).

5. **PLANNING -> TESTING_FIRST gate enforcement** in gate.rs at the existing
   path (gate.rs:583). After current checks: if intent_pairs.len() < min_pairs,
   emit gate_decision SSB row reason="intent_pairs_below_threshold", return
   GateDecision::Reject. Emit IntentConfirmed SSB row on pass. AutonomousGate
   path: auto-confirm with confirmed_by=AutonomousGate when pairs present.

6. **MetamorphicRelation + RelationLibrary** in metamorphic_testing.rs.
   MetamorphicRelation enum { Identity, Inverse, PermutationInvariance }.
   MrResult { relation, passed: bool, sample_size: u32, counterexample: Option<String>,
   duration_ms: u64 }. RelationLibrary { enabled: Vec<MetamorphicRelation> };
   from_config(&[String]) parses strings, unknown -> tracing::warn! (no panic).
   Default: [Identity, Inverse]. PermutationInvariance excluded by default.

7. **MetamorphicTestRunner** in metamorphic_testing.rs. Fields: library:
   RelationLibrary, ssb_tx: SsbSender, sample_size: u32.
   run_identity<F>(f, input: &[u8]) -> MrResult: calls f twice; counterexample = hex
   of input on mismatch. run_inverse<F,G>(encode, decode, samples) -> MrResult:
   decode(encode(x))==x per sample; first fail stored as counterexample string.
   run_permutation_invariance<F>(f, inputs) -> MrResult: only when enabled in library.
   run_all(step_id, ...) -> Vec<MrResult>: runs enabled relations; emits MrViolated
   SSB row per failure; no panic; always returns full set.

8. **Wire MetamorphicTestRunner into auditor pipeline** in deterministic_auditor.rs
   after the C-28 Mutation tier hook. Call metamorphic_runner.run_all() when
   [testing.metamorphic] enabled = true. MrViolated rows contribute to the
   AUDITING->MEMORIZING gate score alongside mutation_score; not a standalone block.

9. **Config: [testing.metamorphic]** in elai_cc.toml: `enabled = true`,
   `relations = ["identity", "inverse"]`. Add MetamorphicConfig { enabled: bool,
   relations: Vec<String> } alongside IntentConfig (Task 4).

10. **IntentPair editor dialog in PlanWindow.svelte**. Inline component
    (no separate file). On BabyStepSpec row selection: collapsible panel listing
    intent_pairs (read via existing Tauri invoke pattern — grep PlanWindow.svelte
    for invoke pattern before wiring). Add button: 3-field form (NL intent textarea,
    assertion textarea, baby_step_ref auto-filled). Save calls add_intent_pair(step_id).
    Confirm button: active when pair_count >= 3; calls confirm_intent_pairs(step_id);
    both return Result<(), String> mapped to toast on error.

11. **Tauri commands** in crates/dashboard-v2/src-tauri/src/lib.rs.
    add_intent_pair(step_id: u64, natural_language_intent: String,
    concrete_test_assertion: String) -> Result<(), String>: appends IntentPair to SSB.
    confirm_intent_pairs(step_id: u64) -> Result<(), String>: calls
    IntentPairSet::confirm(), emits IntentConfirmed SSB row; Err if count < min_pairs.
    Both follow B-36 pattern; register in generate_handler![] macro.

12. **Smoke test** (cloud Sonnet 4.6):
    `elai /task --mode cloud "implement and test a string encoder/decoder pair with 3 intent pairs"`.
    (a) Gate blocked initially: SSB gate_decision row reason=intent_pairs_below_threshold.
    (b) After editor adds 3 pairs + confirm: gate_decision Approve + IntentConfirmed pair_count=3.
    (c) MR_inverse on decode(encode(x))==x for 100 random strings: no MrViolated row.
    (d) Adversarial: introduce off-by-one encoder bug; rerun; assert MrViolated row
    relation=inverse + non-empty counterexample.

13. **Decision doc** workflow/reasoning/C-29-decisions.md: (a) why min_pairs=3
    (single pair cannot distinguish intent from accidental alignment; 3 forces
    multi-angle coverage); (b) why permutation_invariance is off by default (false
    positives on non-commutative ops); (c) why MrViolated is non-blocking alone
    (single relation noise — must aggregate with mutation_score at AUDITING gate).

## WIRING_CONTRACT

produces_symbols:
  - IntentPair, IntentPairSet, IntentConfirmation, ConfirmedBy (intent_pair.rs)
  - BabyStepSpec.intent_pairs field (research_tree.rs — serde(default) extension)
  - SsbActionType::IntentConfirmed { step_id, pair_count, confirmed_by }
  - SsbActionType::MrViolated { step_id, relation, counterexample, sample_size }
  - SCHEMA_VERSION 47 bump in ssb.rs
  - MetamorphicRelation enum (Identity, Inverse, PermutationInvariance)
  - MrResult, RelationLibrary, MetamorphicTestRunner (metamorphic_testing.rs)
  - IntentConfig { min_pairs }, MetamorphicConfig { enabled, relations }
  - elai_cc.toml [testing.intent] + [testing.metamorphic] sections
  - PLANNING->TESTING_FIRST gate: intent_pairs_below_threshold reject path
  - MetamorphicTestRunner hook in deterministic_auditor.rs after Mutation tier
  - Tauri commands add_intent_pair + confirm_intent_pairs (lib.rs)
  - PlanWindow.svelte IntentPairEditor dialog + Confirm button
consumes_symbols:
  - BabyStepSpec (research_tree.rs:1667) — gains intent_pairs field
  - GateDecision (gate.rs:25) — PLANNING->TESTING_FIRST path extended
  - SsbActionType enum (ssb.rs:SCHEMA_VERSION 46) — extended to 47
  - DeterministicAuditor pipeline (deterministic_auditor.rs) — MR hook added
  - AutonomousGate dispatch (autonomous_gate.rs) — auto-confirm path for MR
  - SsbSender (ssb.rs) — used by MetamorphicTestRunner for MrViolated emit
  - PlanWindow.svelte existing BabyStepSpec rendering + invoke pattern
  - elai_cc.toml [testing] parent section
reachable_from:
  - elai_cc.toml [testing.intent] min_pairs gate at PLANNING->TESTING_FIRST
  - elai_cc.toml [testing.metamorphic] relations at auditor MR hook
  - PlanWindow.svelte IntentPairEditor (per-step intent pair population + confirm)
  - DeterministicAuditor pipeline run post-Mutation tier
stub_ban:
  - run_inverse() MUST NOT panic on first mismatch; store counterexample and return
  - confirm_intent_pairs() MUST return Err if pair_count < min_pairs; never silent pass
  - gate MUST emit gate_decision SSB row with reason on every reject; silent block forbidden
  - MrViolated SSB emit MUST include non-empty counterexample payload (not empty string)
  - MetamorphicTestRunner MUST read relations from config, not hardcode them
frontend_pair:
  PlanWindow.svelte IntentPairEditor + Tauri commands add_intent_pair,
  confirm_intent_pairs in crates/dashboard-v2/src-tauri/src/lib.rs
observable_effect:
  trigger: elai /task --mode cloud "implement and test a string encoder/decoder
           pair with 3 intent pairs"
  expect_ssb: gate_decision row reason=intent_pairs_below_threshold initially;
              IntentConfirmed row pair_count=3 after editor confirm;
              no MrViolated rows on clean encoder; MrViolated row relation=inverse
              + non-empty counterexample after adversarial encoder bug introduced
  expect_artifact: SCHEMA_VERSION 47 in ssb.rs; [testing.intent] + [testing.metamorphic]
                   in elai_cc.toml; intent_pair.rs + metamorphic_testing.rs present

## DETERMINISTIC_CHECK

```bash
# 1. IntentPair types present
rg "struct IntentPair|struct IntentPairSet|struct IntentConfirmation|enum ConfirmedBy" \
  .elai_cc/crates/orchestrator/src/intent_pair.rs

# 2. BabyStepSpec gains intent_pairs field
rg "intent_pairs" .elai_cc/crates/orchestrator/src/research_tree.rs

# 3. SCHEMA_VERSION 47
rg "SCHEMA_VERSION.*47|47.*SCHEMA_VERSION" .elai_cc/crates/orchestrator/src/ssb.rs

# 4. SSB action types present
rg "IntentConfirmed|MrViolated" .elai_cc/crates/orchestrator/src/ssb.rs

# 5. MetamorphicTestRunner + MetamorphicRelation
rg "struct MetamorphicTestRunner|enum MetamorphicRelation|struct MrResult|struct RelationLibrary" \
  .elai_cc/crates/orchestrator/src/metamorphic_testing.rs

# 6. Gate enforcement: intent_pairs_below_threshold path
rg "intent_pairs_below_threshold" .elai_cc/crates/orchestrator/src/gate.rs

# 7. MetamorphicTestRunner wired into auditor pipeline
rg "metamorphic_runner\|MetamorphicTestRunner" \
  .elai_cc/crates/orchestrator/src/deterministic_auditor.rs

# 8. Config sections in elai_cc.toml
rg '\[testing\.intent\]|\[testing\.metamorphic\]' .elai_cc/elai_cc.toml

# 9. Tauri commands registered
rg "add_intent_pair|confirm_intent_pairs" \
  .elai_cc/crates/dashboard-v2/src-tauri/src/lib.rs

# 10. PlanWindow IntentPairEditor present
rg "IntentPairEditor\|confirm_intent_pairs\|add_intent_pair" \
  .elai_cc/crates/dashboard-v2/src/lib/windows/PlanWindow.svelte

# 11. Build + scoped tests
cargo check --workspace 2>&1 | tail -3
cargo test -p elai_orchestrator -- intent_pair metamorphic 2>&1 | grep "test result"
cargo test -p elai_cc_contracts -- intent 2>&1 | grep "test result"
```

## ACCEPTANCE

- [ ] IntentPair, IntentPairSet, IntentConfirmation, ConfirmedBy in intent_pair.rs;
      IntentPairSet::confirm() sets confirmed=true + timestamp; len() returns count
- [ ] BabyStepSpec.intent_pairs: serde(default) field; existing BabyStepSpec JSON
      deserializes without schema bump (no regression on existing tests)
- [ ] SCHEMA_VERSION bumped to 47; comment block cites C-29 + T-2 + T-3
- [ ] SsbActionType::IntentConfirmed { step_id, pair_count, confirmed_by } + payload
- [ ] SsbActionType::MrViolated { step_id, relation, counterexample, sample_size } + payload
- [ ] Both variants serialize/deserialize correctly in roundtrip test
- [ ] [testing.intent] min_pairs = 3 in elai_cc.toml; IntentConfig struct wired
- [ ] [testing.metamorphic] enabled = true, relations = ["identity","inverse"] in elai_cc.toml;
      "permutation_invariance" absent from default (deny-by-default enforced)
- [ ] PLANNING->TESTING_FIRST gate: rejects when intent_pairs.len() < 3; emits
      gate_decision SSB row reason=intent_pairs_below_threshold; passes when >= 3
- [ ] AutonomousGate path auto-confirms with confirmed_by=AutonomousGate when pairs present
- [ ] MetamorphicRelation enum: 3 variants; RelationLibrary::from_config() skips unknown
      strings with warn! (no panic); PermutationInvariance excluded from default
- [ ] run_inverse() on 100 random samples: all pass for correct encode/decode pair;
      returns MrResult { passed: true, sample_size: 100, counterexample: None }
- [ ] run_inverse() with adversarial encoder bug: returns MrResult { passed: false,
      counterexample: Some(non-empty string) }; MrViolated SSB row emitted
- [ ] MetamorphicTestRunner hooked into auditor pipeline; runs after Mutation tier;
      controlled by [testing.metamorphic] enabled flag
- [ ] Tauri add_intent_pair: appends pair to BabyStepSpec.intent_pairs; returns Err
      if step_id not found
- [ ] Tauri confirm_intent_pairs: returns Err("need N more pairs") when count < min_pairs;
      emits IntentConfirmed SSB row when count >= min_pairs
- [ ] PlanWindow.svelte IntentPairEditor: renders existing pairs; Add form writes via
      add_intent_pair; Confirm button active only when pair_count >= 3
- [ ] Cloud Sonnet smoke test (Task 12): gate blocked initially (gate_decision row);
      unblocked after 3 pairs confirmed (IntentConfirmed row); MR_inverse passes on
      clean encoder; MrViolated row appears on adversarial encoder
- [ ] cargo check --workspace: zero new errors
- [ ] cargo test -p elai_orchestrator -- intent_pair metamorphic: all pass, >= 8 new fns
- [ ] cargo test -p elai_cc_contracts -- intent: all pass
- [ ] Decision doc workflow/reasoning/C-29-decisions.md (3 decisions documented)

## Files touched

- ADD: .elai_cc/crates/orchestrator/src/intent_pair.rs
    (IntentPair, IntentPairSet, IntentConfirmation, ConfirmedBy)
- ADD: .elai_cc/crates/orchestrator/src/metamorphic_testing.rs
    (MetamorphicRelation, MrResult, RelationLibrary, MetamorphicTestRunner)
- MODIFY: .elai_cc/crates/orchestrator/src/research_tree.rs
    (BabyStepSpec.intent_pairs: IntentPairSet field with serde(default))
- MODIFY: .elai_cc/crates/orchestrator/src/ssb.rs
    (SCHEMA_VERSION 47; SsbActionType::IntentConfirmed + ::MrViolated + payloads)
- MODIFY: .elai_cc/crates/orchestrator/src/gate.rs
    (PLANNING->TESTING_FIRST: intent_pairs_below_threshold reject path)
- MODIFY: .elai_cc/crates/orchestrator/src/deterministic_auditor.rs
    (MetamorphicTestRunner hook after Mutation tier)
- MODIFY: .elai_cc/elai_cc.toml
    ([testing.intent] min_pairs=3; [testing.metamorphic] enabled+relations)
- MODIFY: .elai_cc/crates/dashboard-v2/src-tauri/src/lib.rs
    (add_intent_pair + confirm_intent_pairs Tauri commands; generate_handler! registration)
- MODIFY: .elai_cc/crates/dashboard-v2/src/lib/windows/PlanWindow.svelte
    (IntentPairEditor inline dialog; Add + Confirm buttons; Tauri invoke wiring)
- ADD: workflow/reasoning/C-29-decisions.md

## Notes

T-4 (Risk classification engine), T-5 (EquivalentMutantCache), T-6
(RegressionCorpusManager), T-7 (FormalVerifierHook/Kani), T-8 (cryptographic
provenance), T-9 (CiteAudit), T-10 (PRPO hybrid MSI + RHSI), T-11 (Rulebreakers
Paired-Accuracy), T-12 (XGrammar + CRANE), T-13 (FACET R-DAG) deferred to future
C-N or post-launch per task scope.

## Next

C-30 or the next outstanding C-N depending on prioritization.
If C-27 (T-0) or C-28 (T-1 mutation tier) are not yet shipped, they are true
blockers for the gate smoke test and must be resolved before this session's
PLANNING->TESTING_FIRST gate acceptance can be fully verified.
