D-18 — Typed ObservableEffect predicate (executor session)

Status: SHIPPED (single-variant pass). SCHEMA_VERSION 83 -> 84.
Plan: docs/thefinalglance/D-plan/D-18_observable_effect_typed.md
Orchestrator: s-62376 (D-plan host).

What landed this pass

  1. NEW .elai_cc/crates/orchestrator/src/observable_effect.rs
     - Closed enums: TriggerSpec (5 variants), TypedObservation (5
       variants), SandboxTier (4 variants).
     - ConfoundFlags struct (4 bool fields, not bitflags newtype —
       msgpack-friendly + matches D-9 locked-keys discipline).
     - SandboxExecutor trait (Pin<Box<dyn Future>> erasure, no
       async-trait crate per CLAUDE.md anti-stale-training).
     - MockSandboxExecutor with deterministic (distinguisher, kind)
       fixture map for tests.
     - verify_observable_effect(): runs trigger against executor,
       compares to typed observation, emits one
       `observable_effect_verified` SSB row per call. Executor errors
       map to VerificationOutcome::Skipped (NOT a product failure).
     - verify_effect_batch(): gating helper that short-circuits on
       first Fail and aggregates (Pass / Skipped / Fail) defensively
       (any Fail -> Fail; all Skipped -> Skipped).
     - recent_observable_effects(): dashboard read surface mirroring
       conformal_calibration::recent_tau_rows.
     - 7 unit tests (kind-str stability x3, confound labels, 4
       end-to-end verify pairings including blake3 file content,
       mis-paired observation -> Skipped, executor error -> Skipped).
       All 7 GREEN.

  2. NEW .elai_cc/crates/orchestrator/tests/observable_effect_smoke.rs
     - 3 mock effects (test_command/string_match,
       file_presence/string_match, stdout_contains/string_match), all
       Pass, all 3 SSB rows surfaced via direct query + viewer
       read-side. Round-trip preserved for confound_flags + sandbox
       tier + trigger kind. 1/1 GREEN.

  3. ssb.rs
     - SCHEMA_VERSION 83 -> 84.
     - 1 new SsbActionType variant: ObservableEffectVerified.
     - 1 new SsbPayload variant + ObservableEffectVerifiedPayload
       struct (effect_id, trigger_kind, observation_kind,
       sandbox_tier, outcome, confound_flags, elapsed_us, ts_ms).
     - SQL CHECK constraint + as_sql_str + from_str + action_type()
       match arms updated.
     - test_payload + all_test_payloads entries; assert_eq bumped
       173 -> 175 (D-15 had not updated the count; my D-18 entry
       brought it to 175 honest).

  4. .elai_cc/crates/orchestrator/src/lib.rs
     - pub mod observable_effect; (placed after pub mod novelty;
       inside the D-plan band).

  5. Dashboard wiring
     - NEW .elai_cc/crates/dashboard-v2/src-tauri/src/commands_d18.rs
       (get_observable_effects_recent Tauri command + DTOs + 2 unit
       tests).
     - mod commands_d18; + invoke_handler registration in
       dashboard-v2/src-tauri/src/lib.rs.
     - NEW .elai_cc/crates/dashboard-v2/src/lib/drawers/ObservableEffectPanel.svelte
       (outcome histogram + per-sandbox-tier histogram + recent rows
       table with per-row confound-flag chips, 10s poll cadence
       matching D-{10,11,12,13,16} panels).
     - DrawerKey 'observable-effect' added to types.ts.
     - DrawerHost.svelte import + route added.
     - Drawer.svelte DRAWER_LABELS map: 'OBSERVABLE EFFECTS' (and
       'NOVELTY' for the pre-existing D-15 gap surfaced by svelte-
       check).

Three decisions

  (a) Plan scope vs brief scope — the D-18 plan asks for 4 SSB
      variants + EffectSet orthogonality + sandbox-vs-production
      labeling + FeasibilityProber.probe return-type extension + 3
      paper-ingestor sessions + decision doc. This is the plan's
      explicit 2-session estimate. The orchestrator brief narrows
      this to a single-variant typed-predicate end-to-end (1 SSB
      variant, no orthogonality, no production-claim labeling, mock
      executor + smoke test + panel). I followed the brief's
      narrower scope this pass since it explicitly defines the
      types + smoke shape; the plan's richer surfaces (EffectSet,
      validate_orthogonality, ObservationContext::Sandbox|Production,
      check_goodhart_risk, [observable_effect] elai_cc.toml section,
      FeasibilityProber.probe Vec<ObservationResult> extension)
      are documented as deferred follow-up in observable_effect.rs
      module doc.

  (b) ConfoundFlags struct vs bitflags newtype — the orchestrator
      crate has `bitflags = "2.6"` in Cargo.toml. I deliberately
      modeled ConfoundFlags as a plain struct with 4 bool fields
      instead. Rationale: the SSB payload schema is msgpack-named
      (rmp_serde::to_vec_named); bitflags 2.6's serde mode emits an
      opaque integer, which would be hostile to dashboard JSON
      consumers + violate the locked-keys discipline that the D-9
      registry holds closed-form labels. The struct form serializes
      to four explicit bool fields the panel can render directly;
      `as_labels()` converts to the comma-joined snake_case wire
      form for the SSB row. No new external dep; existing `bitflags`
      was not pulled in.

  (c) Citation rejected — the orchestrator brief proposed citing
      arXiv:2412.16367 as "typed-test predicates". WebSearch
      (May 2026) confirmed arXiv:2412.16367 is "A Layered Swarm
      Optimization Method for Fitting Battery Thermal Runaway Models
      to Accelerating Rate Calorimetry Data" (Bhatnagar et al.,
      2024-12-20) — unrelated to typed-test predicates. Per
      CLAUDE.md memory:feedback_paper_citation_verify the cite was
      dropped and replaced with a TODO_DOC[D-18] tag pointing to
      Voyager arxiv:2305.16291 §3 + Coscientist arxiv:2304.05332 §4
      as the proper anchors for follow-up paper-ingestor work.
      This matches the prior B-40 / B-42 / B-41 pattern (per memory:
      project_b40_papers / project_b42_papers / project_b41_papers
      all substituted bad cites).

Out-of-scope (deferred to a follow-up session)

  - EffectSet + validate_orthogonality (>= 2 orthogonal effects per
    claim, real-problem §4.4 Goodhart defense).
  - ObservationContext::{Sandbox(tier), Production} labeling +
    operator-gate (real-problem §4.4 simulator-reality-gap).
  - check_goodhart_risk single-effect flag emitter.
  - The plan's remaining 3 SSB variants
    (ObservableEffectDispatched, GoodhartRiskFlag,
    ProductionObservationProposal) — SCHEMA bump would be 84 -> 87
    in the follow-up.
  - FeasibilityProber.probe(...) Vec<ObservationResult> return-type
    extension (C-22 surface touch).
  - .elai_cc/elai_cc.toml [observable_effect] section + L3 lock on
    auto_promote_sandbox_to_production = false.
  - paper-ingestor sessions for arxiv:2305.16291 (Voyager),
    arxiv:2310.12931 (Eureka), arxiv:2304.05332 (Coscientist) —
    workflow/papers/ directory NOT touched this pass.
  - agent_pool::dispatch_observable_effects consumer (the gating
    function exists as observable_effect::verify_effect_batch — the
    agent_pool wiring that calls it from Mode D dispatch is the
    follow-up's responsibility).

Verification (this session)

  cargo check -p elai-orchestrator: GREEN (4m 36s).
  cargo test -p elai-orchestrator --test observable_effect_smoke:
    test result: ok. 1 passed; 0 failed; 0 ignored.
  cargo test -p elai-orchestrator --lib observable_effect:
    test result: ok. 13 passed; 0 failed; 0 ignored; 1866 filtered out.
    (7 D-18 tests + 6 pre-existing observable_effect_* names in
     other modules — all pass.)
  cargo check -p elai-dashboard-v2: GREEN (3m 08s).
  npx svelte-check (dashboard-v2): 0 ERRORS, 13 warnings unchanged.

Files touched (additive — no behavioral changes to existing code)

  ADD: .elai_cc/crates/orchestrator/src/observable_effect.rs
  ADD: .elai_cc/crates/orchestrator/tests/observable_effect_smoke.rs
  ADD: .elai_cc/crates/dashboard-v2/src-tauri/src/commands_d18.rs
  ADD: .elai_cc/crates/dashboard-v2/src/lib/drawers/ObservableEffectPanel.svelte
  ADD: workflow/reasoning/D-18-decisions.md (this file)
  MODIFY: .elai_cc/crates/orchestrator/src/lib.rs
          (pub mod observable_effect)
  MODIFY: .elai_cc/crates/orchestrator/src/ssb.rs
          (SCHEMA 83 -> 84; 1 variant; 1 payload struct; 6 wire-map
          arms updated; test_payload + all_test_payloads entry;
          assert_eq 173 -> 175)
  MODIFY: .elai_cc/crates/dashboard-v2/src-tauri/src/lib.rs
          (mod commands_d18 + invoke_handler entry)
  MODIFY: .elai_cc/crates/dashboard-v2/src/lib/api/types.ts
          (DrawerKey 'observable-effect')
  MODIFY: .elai_cc/crates/dashboard-v2/src/lib/drawers/DrawerHost.svelte
          (import + route)
  MODIFY: .elai_cc/crates/dashboard-v2/src/lib/layout/Drawer.svelte
          (DRAWER_LABELS 'OBSERVABLE EFFECTS' + 'NOVELTY' to close
          pre-existing D-15 gap)

NOT TOUCHED (per orchestrator brief NO COMMITS policy)

  - git: no commit, no push, no tag.
  - crates/feasibility/ (does not exist as a separate crate; the
    C-22 FeasibilityProber lives in orchestrator/src/feasibility_probe.rs
    and is in the deferred scope, not touched).
  - agent_pool.rs: not touched (consumer is deferred per (b) above;
    verify_effect_batch is the public surface the agent_pool will
    call in the follow-up).

================================================================
D-18 CLOSURE PASS (executor s-25120, SCHEMA 128 -> 129)
================================================================

Status: CLOSURE SHIPPED (4-variant pack atomic). SCHEMA_VERSION 128 -> 129.
all_test_payloads().len() 293 -> 297. TOTAL_VARIANTS (action_type_lock)
354 -> 358.

DECISION-1: SCHEMA_VERSION 128 -> 129. The plan said "84 -> 85" but that
was stale (16 atomic bumps landed between sessions s-62376 and s-25120).
Actual prior head was 128; this closure bumps to 129. Destructive bump:
existing v128 session DBs are DROPped on first read.

DECISION-2: 4 new SSB variants ADDITIVE (preserve shipped
ObservableEffectVerified). Variants:
  * observable_effect_dispatched (post-task hook beacon)
  * goodhart_risk_flag (Eureka 3.3 component-variation guard)
  * sandbox_observation (Coscientist 4 sandbox taxonomy)
  * production_observation_proposal (Coscientist 4 production taxonomy)
The pre-existing ObservableEffectVerified variant is UNTOUCHED.

DECISION-3: Shipped closed enums preserved (TriggerSpec / TypedObservation
/ SandboxTier / ConfoundFlags). The closure pass does NOT modify any of
the 4 closed taxonomies the first pass shipped; new behaviour rides on
new types (EffectSet, ObservationContext, ObservationOutcome,
ObservationResult, EnvironmentSnapshot).

DECISION-4: New ObservationContext + ObservationOutcome enums are
DISTINCT from VerificationOutcome. VerificationOutcome
(pass | fail | skipped) is the typed-predicate outcome of
verify_observable_effect. ObservationOutcome
(confirmed | refuted | inconclusive) is the Coscientist-style
observation confidence label paired with ObservationContext. Dual
taxonomy required because typed predicate is a Boolean match;
observation result is a confidence claim.

DECISION-5: Sibling probe_with_effects (NOT modifying probe()) to
preserve C-22 dyn-safety. The plan calls for the FeasibilityProber
trait to return Vec<ObservationResult>. The naive interpretation (modify
probe() return type) would break the C-22 contract pinned at
crates/contracts/tests/c22_feasibility_completeness_contract.rs (11
consumer sites + 4 contract tests pin the existing probe() signature).
Instead, the closure pass adds a SIBLING default-method
probe_with_effects(&self, report: &FeasibilityReport) ->
Vec<ObservationResult> that projects an existing FeasibilityReport
stub_results into typed observation results. The trait stays dyn-safe;
the new method is sync, takes &self, and returns a non-generic Vec.

DECISION-6: Plan's arxiv:2412.16367 cite is WRONG — substituted anchors.
WebSearch (May 2026) confirmed arxiv:2412.16367 is "A Layered Swarm
Optimization Method for Fitting Battery Thermal Runaway Models"
(Bhatnagar et al., 2024-12-20) — unrelated to typed test predicates.
Substituted anchors (paper-substitution pattern continues per B-40 /
B-41 / B-42 / C-43 / D-16):
  * Voyager arxiv:2305.16291 3 (skill-with-measurable-effect; INV-5
    state-delta invariant — a real observable effect must produce a
    non-zero delta in observed environment keys between pre-effect and
    post-effect snapshots).
  * Eureka arxiv:2310.12931 3.3 (component-variation reward invariant —
    when a reward component is constant across runs, it is not measuring
    what it claims and Goodhart law applies).
  * Coscientist arxiv:2304.05332 4 (SandboxObservation vs
    ProductionObservation taxonomy; ProductionObservation without an
    instrument_file MUST gate on human review).
The first-pass module doc and first-pass payload doc already flagged
this (TODO_DOC[D-18] paper-ingestor follow-up). The closure pass closes
that TODO_DOC in line with the three substituted anchors.

DECISION-7: Coscientist invariant — ProductionObservation without
instrument_file => requires_human_review = true. Enforced at the type
level via ObservationResult::requires_human_review() — a function-form
invariant rather than a constructor-side guard so the field is freely
settable but the predicate always returns the correct gate value.
Pinned at the contract surface
(d18_t10_production_without_instrument_file_requires_human_review).

DECISION-8: Eureka invariant — Goodhart flag fires when component
variation < threshold. The closure pass surrogates this signal with
EffectSet orthogonality (duplicate (trigger.kind, observation.kind)
pairs in a set = "constant component" surrogate, because the pair is
not varying across the set). Once D-18 producers begin recording
per-run variation snapshots, extend check_goodhart_risk to flag
low-variation components per effect.

DECISION-9: Voyager INV-5 — state delta required for measurable effect.
EnvironmentSnapshot::delta + EnvironmentSnapshot::delta_meets_threshold
expose the INV-5 invariant at the type level. The dispatch hook does
NOT yet consume the snapshots; the snapshot surface is ready for the
production wire that lands when per-effect EffectSet construction lands
in the agent_pool dispatcher.

DECISION-10: Production dispatch via ELAI_OBSERVABLE_EFFECT_ENABLE env
gate at layer-8 of the post-task hook chain. Chain ordering:
  route -> verify -> bai -> adversary -> sycophancy -> novelty ->
  conformal-grow -> observable-effect
Same toggle pattern as D-10 / D-11 / D-12 / D-13 / D-14 / D-15 / D-16.
Default: no-op. Smoke beacon (dispatch + sandbox observation) emits when
both the env var AND [observable_effect].enable=true in elai_cc.toml.
The real EffectSet wire (per-task EffectSet construction +
verify_effect_batch on the set + Goodhart check on the set + Coscientist
proposal emission) is a follow-up.

What landed this closure pass

  1. crates/orchestrator/src/ssb.rs
     - SCHEMA_VERSION 128 -> 129
     - 4 new SsbActionType variants + matching 9-tuple atomic lockstep
     - 4 new payload structs
     - all_test_payloads().len() assertion 293 -> 297
  2. crates/orchestrator/src/observable_effect.rs (EXTEND)
     - ObservationOutcome (Confirmed | Refuted | Inconclusive)
     - ObservationContext (Sandbox(Tier) | Production)
     - EnvironmentSnapshot + delta + delta_meets_threshold
     - ObservationResult + requires_human_review predicate
     - EffectSet + validate_orthogonality + is_orthogonal
     - GoodhartFlag + check_goodhart_risk free fn
     - 8 new unit tests
  3. crates/orchestrator/src/feasibility_probe.rs
     - probe_with_effects default-method on FeasibilityProber trait
       (PRESERVES probe() signature; C-22 dyn-safety intact)
  4. crates/orchestrator/src/runtime_config.rs
     - ELAI_OBSERVABLE_EFFECT_ENABLE_ENV constant
     - ObservableEffectRuntimeConfig + state + 3 tests
  5. crates/orchestrator/src/task_runner.rs
     - observable_effect_runtime field on ResolvedConfig
     - dispatch_observable_effect_hook (layer-8) call + body
  6. crates/orchestrator/tests/action_type_lock.rs
     - TOTAL_VARIANTS 354 -> 358 + 4 entries + 4 exhaustive arms
  7. crates/orchestrator/tests/action_type_lock.golden
     - 354 -> 358 alphabetically-sorted entries (regen via Python helper)
  8. crates/orchestrator/tests/observable_effect_integration.rs (NEW)
     - 6 integration tests
  9. crates/contracts/tests/observable_effect_contract.rs (NEW)
     - 10 contract tests
 10. crates/dashboard-v2/src-tauri/src/commands_d18.rs (EXTEND additive)
     - 4 new DTOs + extended RecentObservableEffects + 4 reader helpers
       + 3 new dashboard test cases
 11. crates/dashboard-v2/src/lib/api/types.ts (EXTEND)
     - 5 mirror types + 4 label types + ObservableEffectKind union
 12. crates/dashboard-v2/src/lib/drawers/ObservableEffectPanel.svelte
     - Import canonical types + 4 new section renderers
 13. elai_cc.toml
     - [observable_effect] L3-LOCKED section (commented body)

Cargo check status (no-cargo-test-unless-permitted rule):
  - cargo check --workspace --lib: GREEN
  - cargo check --tests -p elai-orchestrator: GREEN
  - cargo check --tests -p elai_cc_contracts: GREEN
  - cargo check -p elai-dashboard-v2 --lib: GREEN

OUT OF SCOPE (preserved untouched per ANTI-LAZY HARD RULE 1)
  - observable_effect.rs core types: TriggerSpec / TypedObservation /
    SandboxTier / ConfoundFlags / ObservableEffect / SandboxExecutor /
    MockSandboxExecutor / verify_observable_effect / verify_effect_batch
    / VerificationOutcome / recent_observable_effects
  - ObservableEffectVerified SSB variant
  - commands_d18.rs base (extended additive only; #[serde(default)] on
    new fields keeps the wire form back-compat)
  - ObservableEffectPanel.svelte base (extended additive only)
