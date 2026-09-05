C-29 Decisions — Testing-Trust Tier T-2 (IntentPair) + Tier T-3 (basic Metamorphic relations)

Session: s-25672
Date: 2026-05-19
Branch: main
Spec: docs/thefinalglance/C-plan/C-29_testing_trust_t2_t3_intent_metamorphic.md
Context package: workflow/sessions/s-25672/context-package-C29.md (13 DRIFT items)


DECISION-D1 — SCHEMA_VERSION bump 60 -> 61

Plan body says SCHEMA_VERSION 47 (DRIFT-C29-1). The const at ssb.rs:404 is
60 (post-C-28). C-29 adds two new beacons (IntentConfirmed + MrViolated),
so the bump is 60 -> 61, atomic 9-tuple lockstep:
  * CHECK constraint (CREATE_SSB_TABLE) — added 'intent_confirmed',
    'mr_violated' after 'mutation_survived'.
  * SsbActionType enum — IntentConfirmed + MrViolated variants.
  * as_sql_str() arms — Self::IntentConfirmed => "intent_confirmed", etc.
  * ssb_action_type_from_str() arms — reverse mapping.
  * SsbPayload enum — IntentConfirmed(IntentConfirmedPayload) +
    MrViolated(MrViolatedPayload) variants.
  * SsbPayload::action_type() arms — discriminant mapping.
  * Payload structs — IntentConfirmedPayload { step_id, pair_count,
    confirmed_by } + MrViolatedPayload { step_id, relation, counterexample,
    sample_size } (Debug/Clone/PartialEq/Eq/Serialize/Deserialize).
  * test_payload() + all_test_payloads() — added 2 fixtures.
  * Count assertion 124 -> 126; comment string appends
    "+ 2 C-29 (IntentConfirmed/MrViolated)".

Contract test pin uses floor (`>= 61`) per C-28 DECISION-D13 precedent so
future bumps don't break this assertion.


DECISION-D2 — min_pairs = 3 default

Spec ref: docs/specs/guide/testing-trust-design.txt L567 ("require SSB
action INTENT_CONFIRMED with at least 3 pairs per baby step"). Single-pair
grounding cannot distinguish intent from accidental alignment; three pairs
force multi-angle coverage without ballooning per-step setup cost. The
testing-trust spec L555-558 calls for 3-5 pairs; we pick the lower bound
as the gate floor and allow projects to opt into stricter thresholds via
`[testing.intent].min_pairs`. `min_pairs = 0` disables the gate (developer
ergonomics; not the default).


DECISION-D3 — MrViolated is non-blocking, score aggregation deferred to T-4

The C-29 plan §13(c) explicitly states "MrViolated is non-blocking alone"
because single-relation noise must aggregate with mutation_score at the
AUDITING gate. Per DRIFT-C29-13 we adopt Option A: ship MrViolated row
emission only; defer the AUDITING -> MEMORIZING score-aggregation gate
to T-4 (Risk classification engine). The DeterministicAuditor Step 7
hook bumps `checks_run = 7` when MetamorphicConfig.enabled is true and
no prior Block has fired; per-BabyStep MR corpus integration is also a
T-4+ task (until then the hook records its dispatch without false
positives — clean path emits zero findings).


DECISION-D4 — PermutationInvariance opt-in (default off)

Spec L605-606 notes permutation_invariance only holds for commutative
aggregations; applying it to non-commutative ops produces false
positives. `RelationLibrary::default()` therefore enables only Identity
and Inverse. `[testing.metamorphic].relations` defaults to
`["identity", "inverse"]`. Operators wanting MR_permutation_invariance
add `"permutation_invariance"` to the list explicitly. The runner's
`run_permutation_invariance` is a no-op (returns a passed 0-sample result)
when the relation is not in the enabled set, so callers can always invoke
it and observe deterministic skip behaviour.

Unknown relation names emit `tracing::warn!(target = "metamorphic_testing",
"unknown relation: ...")` and are skipped — never panic, never silently
ignored.


DECISION-D5 — IntentPair field naming (plan over spec verbatim)

Per DRIFT-C29-7, we use the plan's `natural_language_intent` /
`concrete_test_assertion` / `baby_step_ref` instead of the spec's
`input_description` / `expected_output` / `rationale`. Rationale:
  (1) "concrete_test_assertion" makes the Layer-2 grounding role explicit
      (this string is what the Auditor uses to validate Property
      soundness in T-3+); "expected_output" loses that.
  (2) "natural_language_intent" pairs cleanly with the JSON-LD
      intent-grounding pattern future T-4 + T-11 (paired rulebreakers)
      will consume.
  (3) "baby_step_ref" preserves the explicit foreign-key edge to
      BabyStepId — the canonical join key for SSB rows.

Bounded-string lengths follow the spec: 120/120/80 chars (INTENT_MAX_CHARS,
ASSERTION_MAX_CHARS, REF_MAX_CHARS). Constructor validates via
`String::chars().count()` — multi-byte UTF-8 (e.g. emoji) counts as one
char, not 4 bytes (CSQ: a 119-emoji intent passes the 120-char ceiling
even though its byte-count is 476). The contract test
`c29_intent_pair_char_bound_counts_chars_not_bytes` pins this semantic.


DECISION-D6 — IntentPairSet vs. IntentConfirmation are distinct types

Per DRIFT-C29-8. IntentPairSet is per-step state on BabyStepSpec
(`pairs: Vec<IntentPair>`, `confirmed: bool`, `confirmed_at_ms:
Option<i64>`). IntentConfirmation is the SSB-row evidence record
(`step_id: BabyStepId`, `pair_count: usize`, `confirmed_by: ConfirmedBy`,
`timestamp_ms: i64`). `confirm()` on IntentPairSet mutates state +
allows the caller to emit a separate IntentConfirmation. Mirrors the
C-1 GroundingSweepResult / GroundingSweepEvidence pair pattern.

`IntentPairSet::confirm` is idempotent: re-calling preserves the original
timestamp and returns Ok(()).


DECISION-D7 — IntentPairsGate registration (DRIFT-C29-3)

Plan body's `gate.rs:583` insertion is wrong (that's a unit-TEST
assertion). Per DRIFT-C29-3 we add `pub struct IntentPairsGate` in
`evidence.rs` alongside `GroundingSweepGate`, implementing `EvidenceGate`
with `transition() -> (Planning, TestingFirst)`. The gate is appended to
`all_gates()` so order-stable indexing in existing tests stays valid.
`gates_for_transition` already returns every gate whose transition
matches `(source, target)`, so the existing `GroundingSweepGate` and the
new `IntentPairsGate` both fire for PLANNING -> TESTING_FIRST;
`check_transition` short-circuits on the first failure.

The gate default-degrades when `TransitionContext.intent_pairs.is_none()`
(callers that have not yet threaded the field stay backward-compatible)
and when `TransitionContext.min_intent_pairs == 0` (gate disabled).
Production callers populate the field explicitly via the dashboard
write path + the runtime loader threading `[testing.intent].min_pairs`
through.


DECISION-D8 — Autonomous-gate auto-confirm honors the threshold floor

Per DRIFT-C29-3 the autonomous-gate auto-confirm path MUST NOT bypass
the pair-count threshold — that would defeat T-2. We ship the pure
helper `intent_pair::evaluate_autonomous_intent_confirm(&IntentPairSet,
usize) -> Option<usize>`:
  * `Some(count)` when `min_pairs == 0 || set.len() >= min_pairs`.
  * `None` when `set.len() < min_pairs` (auto-confirm refuses; gate
    falls through to the standard Reject path with
    `reason=intent_pairs_below_threshold`).

The full autonomous_gate.rs hook (emitting `gate_decision` Reject rows +
`IntentConfirmed` rows on success) threads `TransitionContext` into the
dispatch site in a follow-up — the helper + contract test
(`c29_autonomous_confirm_honors_threshold_floor`) pin the policy
semantics now.


DECISION-D9 — TransitionContext field extension via Default

Per IMPLICIT-DEP-C29-2 we extend `TransitionContext` with two new fields:
  * `intent_pairs: Option<IntentPairSet>` — None falls through to gate
    passes (back-compat with every existing call site).
  * `min_intent_pairs: usize` — 0 disables the gate.

All existing `TransitionContext { ... }` struct-literal sites use
`..Default::default()` so the additions are non-breaking. The
`#[derive(Default, ..)]` on TransitionContext supplies `None` + `0` for
the new fields. Mirrors the B-38 ambiguity_score / streak_count
extension pattern.


DECISION-D10 — DeterministicAuditor Step 7 (checks_run bump only)

Per DECISION-D3 + DRIFT-C29-13 Option A, Step 7 dispatches the
`MetamorphicTestRunner` but emits no DeterministicFindings on the happy
path (per-BabyStep corpus integration is deferred). `checks_run = 7`
when `metamorphic.enabled && !post_step6_block` is the observable
wire-evidence. Mirrors the C-28 Step 6 short-circuit pattern. Mis-
configured `[testing.metamorphic]` (every relation unknown) surfaces a
single Warn-level DeterministicFinding so the misconfig is observable.
Never emits Block — score aggregation stays evidence-only per
DECISION-D3.


DECISION-D11 — MrViolated counterexample MUST be non-empty (stub_ban)

Per DRIFT-C29-11 + the WIRING_CONTRACT stub_ban clause. Enforcement:
  * `MrResult::failed` debug_asserts non-emptiness and substitutes a
    `"<empty>"` sentinel as a runtime guard so the SSB writer is never
    handed an empty string.
  * `MrViolatedPayload` carries `counterexample: String` directly (no
    Option) so the schema itself forbids absence.
  * Contract test `c29_mr_violated_payload_serde_roundtrip` asserts
    `!back.counterexample.is_empty()` after a round-trip.


DECISION-D12 — IntentPair editor is the first live Plan -> Tauri WRITE path

Per DRIFT-C29-10. PlanWindow.svelte historically renders BabyStepSpec
read-only (focus_tunnel chip poll + bubble_up events poll). The C-29
IntentPairEditor introduces:
  * `add_intent_pair(step_id, natural_language_intent,
    concrete_test_assertion)` — appends a single IntentPair to the
    per-step set held in `AppState.intent_pairs`.
  * `confirm_intent_pairs(step_id, min_pairs?)` — flips
    `IntentPairSet::confirmed` and emits one `IntentConfirmed` SSB row
    via `state.writer`. Returns Err when count < min_pairs.
  * `get_intent_pairs(step_id)` (IMPLICIT-DEP-C29-1) — read-back for
    the editor poller.

AppState gains `intent_pairs: Arc<Mutex<BTreeMap<u64, IntentPairSet>>>`
(BTreeMap per CLAUDE.md coding standards; `std::sync::Mutex` since the
guard is never held across `.await`). `AppState::get_baby_step_intent_pairs`
is the typed accessor.

The dashboard does not yet thread state.intent_pairs back into the
orchestrator's live BabyStepSpec (the navigator tree lives elsewhere).
Until that wire lands, the editor + SSB row + IntentPairsGate threading
through `TransitionContext.intent_pairs` is the canonical evidence
trail; the BabyStepSpec.intent_pairs field acts as the on-tree state
once the orchestrator's task_runner reads from AppState.


DECISION-D13 — Contract test pinned with floor semantics

Per DRIFT-C29-12 + C-28 DECISION-D13 precedent. The contract test
`c29_schema_version_public_floor_is_61` asserts `>= 61` (not `== 61`)
so future SCHEMA bumps stay test-compatible. File location:
`.elai_cc/crates/contracts/tests/c29_intent_pair_metamorphic_contract.rs`.
21 pins shipped covering: SSB schema floor, both new variant string
round-trips, both payload serde round-trips, IntentPair char bounds
(incl. multi-byte CSQ), IntentPair empty-string rejection, IntentPairSet
default, IntentConfig default min_pairs, ConfirmedBy snake_case strings,
RelationLibrary default exclusion of permutation, RelationLibrary
from_config unknown-skip, parse_relation round-trip, MetamorphicConfig
default relations, run_inverse non-empty counterexample on adversarial
encoder, IntentPairsGate registration, gate reject below threshold,
gate pass at threshold, gate default-degrade when threshold zero,
autonomous-confirm threshold floor.


DECISION-D14 — No new Cargo dependencies

Per Anti-Slopsquatting + the C-29 plan stub_ban. The orchestrator crate
does NOT depend on `chrono`, so IntentPairSet timestamps use
`i64` ms since UNIX_EPOCH (`std::time::SystemTime::duration_since`) —
matches the existing Ssb pattern. No new pyproject.toml or Cargo.toml
deps required.


Test results (orchestrator + contracts, scoped to C-29 surface)

  cargo check --workspace                                  PASS (zero errors)
  cargo test -p elai-orchestrator --lib intent_pair        12 PASS / 0 FAIL
  cargo test -p elai-orchestrator --lib metamorphic_testing 13 PASS / 0 FAIL
  cargo test -p elai-orchestrator --lib ssb::tests::test_typed_payloads  1 PASS
  cargo test -p elai_cc_contracts --test c29_intent_pair_metamorphic_contract
                                                            21 PASS / 0 FAIL

Total new tests this session: 47 PASS / 0 FAIL.


Files added

  .elai_cc/crates/orchestrator/src/intent_pair.rs
  .elai_cc/crates/orchestrator/src/metamorphic_testing.rs
  .elai_cc/crates/contracts/tests/c29_intent_pair_metamorphic_contract.rs
  workflow/reasoning/C-29-decisions.md

Files modified

  .elai_cc/crates/orchestrator/src/lib.rs
  .elai_cc/crates/orchestrator/src/research_tree.rs
  .elai_cc/crates/orchestrator/src/ssb.rs
  .elai_cc/crates/orchestrator/src/evidence.rs
  .elai_cc/crates/orchestrator/src/deterministic_auditor.rs
  .elai_cc/crates/orchestrator/src/deterministic_audit_gate.rs
  .elai_cc/crates/orchestrator/src/focus_tunnel/pre_assembly.rs
  .elai_cc/crates/orchestrator/src/pipeline_tree_navigator.rs
  .elai_cc/crates/orchestrator/src/task_runner.rs
  .elai_cc/crates/cli/src/runtime_config.rs
  .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs
  .elai_cc/crates/dashboard-v2/src-tauri/src/lib.rs
  .elai_cc/crates/dashboard-v2/src-tauri/src/state.rs
  .elai_cc/crates/dashboard-v2/src/lib/windows/PlanWindow.svelte
  .elai_cc/elai_cc.toml


Deferred to follow-up

  * Autonomous-gate `decide()` integration site (the helper exists; the
    full call-site threading of TransitionContext.intent_pairs through
    `GateContext` -> `AutonomyDecision` is a wave-2 wiring task).
  * AppState.intent_pairs -> orchestrator-side BabyStepSpec.intent_pairs
    propagation (the navigator tree is held in task_runner; this is the
    same wave as the autonomous-gate integration).
  * Per-BabyStep MR corpus integration in DeterministicAuditor Step 7
    (T-4+).
  * AUDITING -> MEMORIZING score aggregation including mutation_score
    + MrViolated row counts (T-4 Risk classification engine).
  * Cloud Sonnet 4.6 end-to-end smoke test from the plan §12 (requires
    live sidecar; deferred until session has explicit HARDWARE-HEAVY
    approval).


END
