PHASE 2C AUDIT — Per-Agent GRC Configuration
Auditor mode: HOSTILE. Evidence-first verdicts.

================================================================
SUMMARY (TL;DR)
================================================================
Overall verdict: IMPLEMENTED (with two notable integration gaps).

Core reasoning-crate deliverables (2C.1–2C.5) are real, compiling, tested, and
all 15 tests pass. Orchestrator consumes AgentGrcConfig at spawn time and calls
with_agent_config on a fresh engine per handle. Phase 2C metrics file exists
with all required fields. Anti-rubber-stamp is wired and enforced in
engine.rs.

Gaps found:
  G1. CONFIG LAYER NOT ACTUALLY TOML-DRIVEN. TomlConfigLoader in
      crates/reasoning/src/config.rs:34-188 does NOT read from elai_cc.toml.
      It is a hardcoded-match stub with per-role constants. elai_cc.toml
      contains zero [grc.defaults] or [agents.{role}.grc] tables (only
      [agents.auditor] with deterministic_first, unrelated to 2C).
      Spec D3 / 2C.4 I5 says load from core::Config / elai_cc.toml. This is
      deferred to phase 4A per the loader's own comment ("In phase 4A, these
      overrides come from core::Config parsing"). Acceptable as a stub per D3
      "reasoning/ receives parsed AgentGrcConfig from orchestrator", but the
      deliverable is a policy-embedded trait implementation, not a
      configuration loader. EXIT CRITERIA phrase "Global defaults + per-role
      overrides merge correctly (field-by-field, not deep)" passes for the
      in-memory struct layer only — there is no evidence of end-to-end TOML
      override, only the `_ => defaults` fallback branch.
  G2. TWO SOURCES OF TRUTH FOR ROLE DEFAULTS. Both
      crates/reasoning/src/config.rs::TomlConfigLoader::load_config AND
      crates/orchestrator/src/agent_pool.rs::default_grc_config hardcode
      per-role AgentGrcConfig. They DISAGREE:
        - Designer: reasoning lists [Observe, Hypothesize, PlanAction, Verify,
          Converge] + override PlanAction->Verify; orchestrator lists
          [Observe, PlanAction, Act, Verify] with NO override. Orchestrator's
          chain is INVALID under base allowed_next (Observe cannot go to
          PlanAction directly — Observe.allowed_next = [Hypothesize, Converge,
          Status]). Orchestrator version never passes through validate_config.
        - PromptEngineer: reasoning uses budget (5,800) + [Observe,
          Hypothesize, Converge]; orchestrator uses (5,800) + full 5-step
          chain [Observe, Hypothesize, PlanAction, Act, Verify]. Spec T13
          matches reasoning's chain.
        - ContextCompiler: reasoning uses [Observe] only, High grounding,
          FILE_CHECK+SOURCE_CHECK only; orchestrator uses ..default() =
          5-step chain, Uncertain grounding, all 6 methods. This is the
          EXIT-CRITERIA-defining divergence: "Context-compiler gets
          observe-only (hypothesize/plan_action/act/verify rejected)" is NOT
          enforced when orchestrator spawns a ContextCompiler agent in
          production wiring. Only the reasoning-crate tests verify the
          observe-only invariant. Orchestrator test file at agent_pool.rs
          does not guard ContextCompiler's required_steps.

6 roles all present: YES (executor, auditor, context-compiler, memory-writer,
  prompt-engineer, designer) in reasoning/src/config.rs. Plus Researcher and
  MediaValidator in orchestrator (8 total). Budgets per spec: all match at
  the reasoning layer; orchestrator layer has divergences per G2.

Anti-rubber-stamp wired: YES, in engine.rs:297-315. Converge by role=auditor
  requires >=1 Verify step with "FINDING:" prefix (case-sensitive, after
  trim_start). Rejected via GrcError::InsufficientFindings. Test t07 passes.

TOML file holding config: NONE. No elai_cc.toml section defines GRC
  per-role config. Defaults live in Rust code (two places, see G2). The
  project's config loader is a trait stub awaiting phase 4A TOML wiring.

================================================================
PER-SUB-STEP VERDICTS
================================================================

2C.1 SKELETON: AgentGrcConfig type + config loading stubs — PASS
-------------------------------------------------------------
Evidence:
  crates/reasoning/src/types.rs:193-221 — AgentGrcConfig with 8 public fields:
    role, budget, required_steps, grounding_level, csq_required,
    allowed_methods, transition_overrides, require_scope_enumeration.
  types.rs:168-172 — GrcError variants InsufficientFindings and
    ConfigValidation both defined with expected shape.
  crates/reasoning/src/config.rs:9-27 — AgentGrcConfigLoader trait with
    load_config(role, &GrcDefaults) and validate_config(&AgentGrcConfig).
  config.rs:34-46 — TomlConfigLoader struct + Default + new().
  crates/reasoning/src/engine.rs:43-45 — GrcEngine trait has
    with_agent_config(&mut self, AgentGrcConfig) and get_agent_config().
  crates/reasoning/src/lib.rs:8-15 — re-exports AgentGrcConfig,
    AgentGrcConfigLoader, TomlConfigLoader, GrcDefaults.
  types.rs:256-279 — GrcDefaults struct with budget, grounding_level,
    csq_required, allowed_methods. Default impl provided.

Verdict: All DETERMINISTIC_CHECK items in the spec are satisfied.

2C.2 VERIFY: compilation check — PASS
-------------------------------------------------------------
Evidence:
  cargo test -p elai-reasoning --test agent_config: Finished test profile in
    1.45s with 0 errors. Workspace compiles clean.
  reasoning/Cargo.toml does not pull in a toml crate (TOML parsing lives in
    core/). Confirmed by inspection — loader is hardcoded match, needs no
    serde_toml.

Verdict: PASS.

2C.3 TEST: write failing tests (TDD) — PASS
-------------------------------------------------------------
Evidence:
  crates/reasoning/tests/agent_config.rs — 15 tests named t01_..t15_,
    enumerated exactly 1:1 against spec T1-T15:
      t01 auditor_gets_double_verify
      t02 context_compiler_observe_only
      t03 executor_budget_8_1200
      t04 memory_writer_budget_2_400
      t05 config_validation_empty_steps_rejected
      t06 config_validation_budget_too_small
      t07 auditor_anti_rubber_stamp
      t08 auditor_rejects_uncertain_grounding
      t09 context_compiler_allowed_methods
      t10 global_defaults_with_per_role_override
      t11 transition_overrides_are_additive
      t12 auditor_budget_10_1200
      t13 prompt_engineer_budget_5_800
      t14 designer_config_distinct_from_executor
      t15 require_scope_enumeration_per_role
  grep -c "#[test]" → 15. Meets the >=15 floor.
  Live run result: 15 passed; 0 failed.

Adversarial concerns addressed:
  T7: uses FINDING_PREFIX prefix match with trim_start (engine.rs:305) —
    matches spec semantics.
  T11: test verifies default transitions PRESERVED alongside overrides
    (line numbers in test confirm both Hypothesize default AND new
    override appear).

Verdict: PASS.

2C.4 IMPLEMENT: config loading, per-role routing, transition extension — PASS
-------------------------------------------------------------
I1 validate_config: config.rs:190-239 implements all 5 rules (a)-(e) from D2.
  Non-empty, valid chain, budget>=steps, csq requires Verify, allowed_methods
  subset check.
I2 transition merging + whitelist: engine.rs:122-142 effective_transitions().
  Union of base.allowed_next() with config.transition_overrides (additive
  only, via push-if-absent in config.rs:248-262). Filter by whitelist =
  required_steps + [Status, Converge]. Correctly prevents
  context-compiler (required_steps=[Observe]) from leaking Hypothesize.
I3 grounding level enforcement: engine.rs:381-390 rejects if level <
  config.grounding_level. types.rs:125-133 GroundingLevel derives PartialOrd
  with CORRECT variant order Uncertain=0, High=1, Verified=2 (the phase 2A
  gap identified in spec L974-977 has been fixed).
I4 anti-rubber-stamp: engine.rs:18 const FINDING_PREFIX = "FINDING:".
  engine.rs:297-315 — on Converge where role == "auditor", counts
  state.steps filtered to (step_type == Verify) AND
  (content.trim_start().starts_with(FINDING_PREFIX)). count < 1 →
  InsufficientFindings { role, expected_min: 1 }. Matches spec exactly.
I5 config loading from TOML: PARTIAL. Trait implemented with hardcoded
  per-role match + validate_config call. Does NOT parse elai_cc.toml. See G1.
  Validation IS called before returning (config.rs:186). The fallback for
  unknown roles uses defaults only without error, per spec.
I6 per-role routing + topic reset: engine.rs:581-585 —
  with_agent_config clears self.topics (full reset) and stores config. No
  leftover state.

No todo!/unimplemented!/unwrap() in config.rs or engine.rs (only
  String::new() defaults and proper Option/Result propagation). Confirmed
  by inspection of complete file contents.

Verdict: PASS on in-crate deliverables. I5 TOML parsing is effectively a
  stub pending phase 4A.

2C.5 MEASURE: config overhead and correctness metrics — PASS (with caveat)
-------------------------------------------------------------
Evidence:
  .elai_cc/metrics/phase_2C.toml exists with all required fields:
    config_load_us = 0 (measured in ns because <1us)
    config_load_ns = 400
    routing_overhead_pct = 28.5  <-- FAILS <10% target
    config_stack_bytes = 144, config_total_bytes = 278,
    config_pathological_bytes = 527  (well under 2KB target)
    test_count = 49, test_failures = 0, phase_2c_tests = 15
    source_loc_delta = 530

Caveat: routing_overhead_pct=28.5 exceeds the <10% ACCEPTANCE target.
  The metric file includes an explanatory comment that "absolute overhead
  is ~140ns/step" and "At sub-microsecond scale, percentage is amplified
  by measurement noise. Real-world: LLM calls take 1-30s per step,
  making 140ns overhead unmeasurable in production." This is a rational
  explanation but the spec's deterministic check ("routing_overhead_pct
  value < 10.0") would fail a mechanical gate. Auditor note: this is a
  documented-deviation, not a hidden regression.

Verdict: PASS on file existence and all sub-targets except
  routing_overhead_pct. Deviation documented in-file.

================================================================
WIRING VERIFICATION: orchestrator USES AgentGrcConfig?
================================================================

Yes. Evidence:
  crates/orchestrator/src/agent_pool.rs:60 — AgentHandle.grc_config:
    AgentGrcConfig.
  agent_pool.rs:66 — AgentHandle.grc_engine: BasicGrcEngine (per-handle
    engine instance).
  agent_pool.rs:86-87 — Clone impl creates fresh BasicGrcEngine and calls
    engine.with_agent_config(grc_config.clone()).
  agent_pool.rs:365-371 — spawn() calls default_grc_config(&role) and then
    grc_engine.with_agent_config(grc_config.clone()) on a fresh engine.
  agent_pool.rs:467-592 — default_grc_config matches 8 AgentRole variants
    (Executor, Auditor, Designer, PromptEngineer, MemoryWriter,
    ContextCompiler, Researcher, MediaValidator).
  agent_pool.rs:677-710 — inline tests grc_config_executor,
    grc_config_memory_writer, grc_config_designer_distinct_from_executor
    validate orchestrator's role→config mapping.
  Contract tests: crates/contracts/tests/checkpoint_contract.rs:16-20 and
    recovery_contract.rs:14-18 use AgentGrcConfig.

Divergences (G2 above):
  - orchestrator::default_grc_config(Designer) has required_steps = [Observe,
    PlanAction, Act, Verify] with no transition_overrides. Observe→PlanAction
    is NOT in base allowed_next(Observe) = [Hypothesize, Converge, Status].
    This config would fail validate_step_chain. Orchestrator never calls
    validate_config on its hardcoded mapping — so the engine will accept the
    config and later reject at transition time. Bug-adjacent: the Designer
    agent would fail on its second think() call in production unless an
    override is added. reasoning::config.rs's designer correctly inserts
    PlanAction→Verify and uses [Observe, Hypothesize, PlanAction, Verify,
    Converge] — diverges both in chain and overrides.
  - orchestrator::default_grc_config(ContextCompiler) uses
    ..AgentGrcConfig::default() with only role override. Default gives 5-step
    chain + Uncertain grounding + all 6 methods. This VIOLATES EXIT
    CRITERIA: "Context-compiler gets observe-only
    (hypothesize/plan_action/act/verify rejected)." The production spawn
    of a ContextCompiler via orchestrator will NOT enforce observe-only or
    FILE_CHECK+SOURCE_CHECK-only.
  - orchestrator::default_grc_config(PromptEngineer) uses 5-step chain;
    reasoning's loader uses [Observe, Hypothesize, Converge]. Spec T13
    matches reasoning's.
  - orchestrator::default_grc_config(MemoryWriter) has required_steps
    [Observe, Converge]. reasoning's loader same. OK.
  - orchestrator's Executor and Auditor match spec budgets and
    grounding_level. OK.

This is a reconciliation item: the orchestrator MUST either call the
  reasoning TomlConfigLoader or its local default_grc_config MUST be
  corrected to match the reasoning crate's policy table. Right now the two
  are parallel implementations and they drift.

================================================================
6 ROLES × BUDGET VERIFICATION (against spec PHASE 2C EXIT CRITERIA)
================================================================

Role              Spec budget    reasoning/config.rs    orchestrator/agent_pool.rs
executor          8/1200         8/1200  OK             8/1200  OK
auditor           10/1200        10/1200 OK             10/1200 OK
context-compiler  3/600 observe  3/600   OK             default 5/800  FAIL
memory-writer     2/400          2/400   OK             2/400   OK
prompt-engineer   5/800          5/800   OK             5/800 budget OK,
                                                         wrong chain
designer          8/1200         8/1200  OK             8/1200 budget OK,
                                                         invalid chain

All reasoning-layer policies correct. Orchestrator layer has 3 divergences.

================================================================
ANTI-RUBBER-STAMP ENFORCEMENT (EXIT CRITERIA)
================================================================

Wired: YES.
Location: crates/reasoning/src/engine.rs:18 (FINDING_PREFIX constant) +
  engine.rs:297-315 (check on think() when step_type==Converge and
  role=="auditor").
Test: t07_auditor_anti_rubber_stamp — passes.

Edge cases covered (per spec D5 "exact prefix, leading whitespace, content
  without prefix"): trim_start() before starts_with handles leading
  whitespace. Case-sensitive prefix match is by design (internal protocol).

================================================================
CONFIG-FILE VERIFICATION
================================================================

File: F:\.ELAI_workflow\.elai_cc\elai_cc.toml (289 lines)
Search results:
  grep -n "[grc.defaults]" → no matches
  grep -n "[agents." → 1 match at line 42: [agents.auditor]
    (contains deterministic_first = true — unrelated to 2C GRC budget)

Conclusion: elai_cc.toml does NOT hold any phase 2C GRC configuration. The
  effective config is in Rust code at two sites (see G2). The
  TomlConfigLoader is a trait stub that never reads the TOML file; it
  returns hardcoded per-role AgentGrcConfig values and calls
  validate_config before returning.

Underscore naming convention: N/A for elai_cc.toml (no entries exist). The
  reasoning crate's match arms DO use underscore keys in its hardcoded
  cases: "context_compiler", "prompt_engineer", "memory_writer" — matches
  the spec-mandated underscore convention.

================================================================
REGRESSION / CLEAN-CODE CHECKS
================================================================

unwrap() / todo!() / unimplemented!() in config.rs or engine.rs: none found
  (inspection). Spec acceptance item met.

Phase 2A and 2B regression: not individually rerun by this audit; metrics
  file reports test_count=49 with 0 failures (18 phase-2A + 16 phase-2B +
  15 phase-2C = 49). Matches expected.

GroundingLevel variant order: Uncertain, High, Verified — corrected from
  the phase 2A inversion noted in SPEC GAP #4. PartialOrd derived.
  Comparison Uncertain < High < Verified holds.

================================================================
FINAL VERDICT
================================================================

Phase 2C is IMPLEMENTED at the reasoning crate level with all 15 tests
passing. Spec deliverables 2C.1-2C.5 are substantively present.

Two integration gaps must be flagged before calling this phase DONE:
  G1. TomlConfigLoader does not actually read elai_cc.toml. "Config is
      layered: global defaults + per-role overrides in [agents.{role}.grc]"
      is aspirational — no such TOML tables exist. Defer-to-4A is
      defensible but must be documented.
  G2. Two parallel role→AgentGrcConfig tables exist (reasoning::config.rs
      and orchestrator::agent_pool.rs) and they DISAGREE on Designer,
      PromptEngineer, and ContextCompiler. Orchestrator's ContextCompiler
      default violates an explicit EXIT CRITERION ("observe-only"), and
      Designer's chain is invalid under the GRC transition rules — the
      orchestrator has never validated it. This is a production-path bug,
      not merely a code-smell.

Action items for reconciliation:
  A1. Remove the hardcoded match in orchestrator::default_grc_config and
      delegate to elai_reasoning::TomlConfigLoader::load_config(role, &defaults)
      so there is ONE source of truth.
  A2. Wire core::Config to actually parse [grc.defaults] and
      [agents.{role}.grc] sections, or explicitly scope Phase 2C to
      "policy table in Rust, TOML wiring deferred to 4A" in the summary.
  A3. Re-audit orchestrator grc_config_* tests to assert observe-only for
      ContextCompiler and valid-chain for Designer.
  A4. routing_overhead_pct=28.5% vs <10% target is documented but fails
      mechanical DETERMINISTIC_CHECK. Either (a) update target to
      absolute_overhead_ns<500 or (b) re-measure with longer steps.
