PHASE 2 LOSSLESS SUMMARY (2A through 2D)
Source files:
  F:\.ELAI_workflow\docs\specs\guide\steps\phase-2A.txt
  F:\.ELAI_workflow\docs\specs\guide\steps\phase-2B.txt
  F:\.ELAI_workflow\docs\specs\guide\steps\phase-2C.txt
  F:\.ELAI_workflow\docs\specs\guide\steps\phase-2D.txt
Project root for code: F:\.ELAI_workflow\.elai_cc\crates\reasoning\

================================================================
## PHASE 2A: GRC STATE MACHINE (basic)
================================================================
Prerequisites: Phase 0 complete (all contract tests pass, cargo check clean)
SCOPE: see docs/specs/guide/implementation-guide.txt

GOAL
Implement the Grounded Reasoning Chain engine core. All reasoning must cite grounding sources. Budget system enforces per-topic call and character limits. State machine validates transition order. Rejected steps produce typed errors. This phase covers the GRC state machine, budget system, and grounding validation only. Grounding verification methods (FILE_CHECK, WEB_CHECK, etc.) are phase 2B. Per-agent configuration is phase 2C. CSQ is phase 2D.

READ BEFORE IMPLEMENTING
  docs/specs/crates/reasoning.txt             Sections 1-3 (responsibilities, GRC state machine, budget)
  docs/specs/crates/reasoning.txt             Section 10 (KEY TYPES: ThinkStep, GrcBudget, GrcError, etc.)
  docs/specs/crates/reasoning.txt             Section 11 (TESTABLE INVARIANTS: items 1-3 are phase-2A scope)
  docs/specs/architecture-overview.txt        Section 4 (GrcEngine trait: think, set_budget)
  docs/specs/integration/prompt-architecture.txt  Section 1 (why code-driven, not prompt-driven)

DESIGN DECISIONS (grounded)
  D1. State transition validation via allowed_transitions map.
      observe -> hypothesize -> plan_action -> act -> verify must reject invalid transitions
      (e.g., hypothesize -> verify skipping plan_action). Implementation: GrcState enum with
      fn allowed_next(&self) -> &[GrcState]. Grounding: standard FSM practice; guidance-ai
      (github.com/guidance-ai/guidance) FSM-from-schema. Our version validates at the step
      level, not token level.
  D2. Typed error enum for all rejection modes. GrcError { UngroundedStep, BudgetExhausted,
      InvalidTransition, MalformedGrounding }. Each variant carries context (which step,
      what was missing, what was expected). Grounding: thiserror pattern.
  D3. Budget tracking with forced convergence. GrcBudget tracks remaining_calls and
      remaining_chars, decremented atomically per step. When remaining_calls <= 1, engine
      forces a convergence step (must produce conclusion). Grounding: BudgetThinker
      (OpenReview 2025, openreview.net/forum?id=ahatk5qrmB); Token-Budget-Aware LLM
      Reasoning (ACL 2025).
  D4. Grounding field validation decomposes content into claims. Parses grounding string
      into individual source citations and verifies each is well-formed
      (format: "source_type:reference", e.g., "file:src/foo.rs:L42"). Phase 2B adds
      actual source verification; phase 2A validates format only. Grounding: RARR
      (arxiv:2210.08726); SAFE (arxiv:2403.18802).
  D5. GroundingLevel enum matching spec hierarchy. VERIFIED > HIGH > UNCERTAIN. Steps
      with UNCERTAIN grounding are flagged (warn log) but not rejected in phase 2A.
      Phase 2B adds resolution. Grounding: reasoning.txt section 4.

----------------------------------------------------------------
### 2A.1 SKELETON: reasoning/ crate structure + types
----------------------------------------------------------------
PRE_GATE:
  - cargo check succeeds (Phase 0 baseline clean)
    Command: cd .elai_cc && cargo check 2>&1; [ $? -eq 0 ]
  - reasoning/ crate does not exist yet (greenfield)
    Command: test ! -d .elai_cc/crates/reasoning && echo PASS || echo "FAIL: crate already exists"
TASK:
  - Create .elai_cc/crates/reasoning/ with Cargo.toml (deps: core/)
  - Add reasoning crate to workspace Cargo.toml
  - Define types in reasoning/src/types.rs:
      GrcState enum { Observe, Hypothesize, PlanAction, Act, Verify, Converge, Status }
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        with fn allowed_next(&self) -> &[GrcState] returning valid transitions
        Note: Converge allowed from any state. Status allowed from any state.
        First non-Status/non-Converge step in a topic must be Observe.
      ThinkStep { step_type: GrcState, content: String, topic: String,
                  grounding: Option<String>, conclusion: Option<String>,
                  scope: Option<Vec<String>> }
        #[derive(Debug, Clone)]
        scope: when AgentGrcConfig.require_scope_enumeration=true, the Observe step must
        populate scope with all task components/modalities before proceeding. Orchestrator
        validates: every scope item appears in subsequent plan_action. Unaddressed items
        -> WARN injected into agent context. Source: reasoning.txt section 10,
        technical-analysis-v3.txt Concern 3.
      ThinkResult { step: ThinkStep, budget_remaining: GrcBudget,
                    grounding_level: Option<GroundingLevel>, warnings: Vec<String> }
        #[derive(Debug, Clone)]
      GrcBudget { max_calls: u32, max_chars: u32, calls_used: u32, chars_used: u32 }
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        fn remaining_calls(&self) -> u32, fn remaining_chars(&self) -> u32,
        fn is_exhausted(&self) -> bool, fn must_converge(&self) -> bool (remaining_calls <= 1)
        Default: max_calls=5, max_chars=800 (reasoning.txt s3 "moderate" default)
      GroundingLevel enum { Verified, High, Uncertain }
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
      GrcError enum { UngroundedStep{step_type,topic},
                      BudgetExhausted{topic,calls_used,max_calls},
                      InvalidTransition{from: GrcState, to: GrcState, allowed: Vec<GrcState>},
                      MalformedGrounding{raw,reason} }
        #[derive(Debug, Clone)] implementing std::error::Error via thiserror
        Note: InvalidTransition carries Vec<GrcState>, requires GrcState: Clone.
  - Define GrcEngine trait in reasoning/src/engine.rs:
      Phase 2A methods (implemented):
        fn think(&mut self, step: ThinkStep) -> Result<ThinkResult, GrcError>
        fn set_budget(&mut self, topic: &str, max_calls: u32, max_chars: u32)
        fn get_budget(&self, topic: &str) -> Option<&GrcBudget>
        fn reset_topic(&mut self, topic: &str)
      Phase 2B stubs (unimplemented!(), present for trait stability):
        fn verify_claim(&self, claim: &str, source: &str) -> Result<VerificationResult, GrcError>
        fn ground_plan(&self, plan: &str) -> Vec<GroundingResult>
      Rationale: defining the full 6-method trait now prevents breaking BasicGrcEngine impls
      when phase 2B adds verify_claim/ground_plan. Stubs use unimplemented!() with
      #[allow(unused)] until phase 2B.
  - Define stub types for 2B forward references in reasoning/src/types.rs:
      VerificationResult { claim: String, passes: bool, evidence: String, method_used: String }
      GroundingResult { claim: String, method: String, level: GroundingLevel, source: String }
      Both marked #[allow(dead_code)] until phase 2B.
  - Stub implementation: BasicGrcEngine with HashMap<String, TopicState>
    where TopicState { budget: GrcBudget, last_state: Option<GrcState>,
                       steps: Vec<ThinkStep>, grounding_log: Vec<(String, GroundingLevel)> }
    Default budget applied when think() is called on a topic with no prior set_budget().
  - Re-export public API from reasoning/src/lib.rs
DETERMINISTIC_CHECK:
  - cargo check on workspace succeeds with 0 errors
    Command: cd .elai_cc && cargo check 2>&1 | grep -c "error\[" | grep -q "^0$"
  - All types exist in source:
    Command: grep -c "pub enum GrcState" .elai_cc/crates/reasoning/src/types.rs | grep -q "1"
    Command: grep -c "pub struct ThinkStep" .elai_cc/crates/reasoning/src/types.rs | grep -q "1"
    Command: grep -c "pub struct ThinkResult" .elai_cc/crates/reasoning/src/types.rs | grep -q "1"
    Command: grep -c "pub struct GrcBudget" .elai_cc/crates/reasoning/src/types.rs | grep -q "1"
    Command: grep -c "pub enum GrcError" .elai_cc/crates/reasoning/src/types.rs | grep -q "1"
    Command: grep -c "pub enum GroundingLevel" .elai_cc/crates/reasoning/src/types.rs | grep -q "1"
    Command: grep -c "pub trait GrcEngine" .elai_cc/crates/reasoning/src/engine.rs | grep -q "1"
  - GrcState derives Clone (required for InvalidTransition variant):
    Command: grep -B1 "pub enum GrcState" .elai_cc/crates/reasoning/src/types.rs | grep -q "Clone"
ADVERSARIAL:
  - If GrcState::allowed_next() is wrong (e.g., allows Observe->Verify), invalid reasoning
    chains pass silently. Test: unit test that Observe.allowed_next() does NOT contain Verify.
  - If GrcBudget uses signed integers, underflow possible. Test: assert remaining_calls()
    returns 0 when calls_used >= max_calls, never wraps to u32::MAX.
ACCEPTANCE:
  - cargo check succeeds with reasoning/ in workspace: exit code 0
  - GrcEngine trait has exactly 6 methods (4 implemented + 2 phase-2B stubs)
    Command: grep -A1 "pub trait GrcEngine" .elai_cc/crates/reasoning/src/engine.rs | head -50 | grep -c "fn "
    (should be 6: think, set_budget, get_budget, reset_topic, verify_claim, ground_plan)

----------------------------------------------------------------
### 2A.2 VERIFY: compilation check
----------------------------------------------------------------
PRE_GATE:
  - Step 2A.1 complete (reasoning/ crate exists and compiles)
    Command: test -f .elai_cc/crates/reasoning/Cargo.toml && echo PASS
TASK:
  - Run cargo check on full workspace. Fix any compile errors.
  - Verify reasoning/ depends only on core/ (no other crate deps for phase 2A).
DETERMINISTIC_CHECK:
  - cargo check returns 0 errors
    Command: cd .elai_cc && cargo check 2>&1 | grep "error\[" | wc -l | grep -q "^0$"
  - Dependency check:
    Command: grep "^\[dependencies" .elai_cc/crates/reasoning/Cargo.toml -A 20 | grep -v "core" | grep -v "^\[" | grep -v "^$" | grep -v "thiserror" | grep -v "serde" | wc -l
    (should be 0 - only core, thiserror, serde allowed)
ADVERSARIAL:
  - If reasoning/ accidentally depends on providers/ or tools/, premature coupling.
    Phase 2A is pure logic; no LLM calls. Test: parse Cargo.toml deps.
ACCEPTANCE:
  - cargo check exit code 0
  - reasoning/ depends only on core/, thiserror, serde (no providers/, tools/, knowledge/)

----------------------------------------------------------------
### 2A.3 TEST: write failing tests (TDD)
----------------------------------------------------------------
PRE_GATE:
  - Step 2A.2 passes (workspace compiles clean)
  - reasoning/src/engine.rs contains GrcEngine trait
    Command: grep -q "pub trait GrcEngine" .elai_cc/crates/reasoning/src/engine.rs
TASK: Write 12 tests in reasoning/tests/ (all must FAIL against stub):
  T1. observe_requires_grounding: think(ThinkStep{step_type:Observe, grounding:None,...})
      => Err(GrcError::UngroundedStep). Invariant: reasoning.txt s11 item 1.
  T2. budget_overflow_forces_convergence: budget(topic="t", max_calls=3, max_chars=2000).
      think() twice non-Converge (uses 2, remaining=1) => must_converge==true. think()
      non-Converge => Err(BudgetExhausted) (forced Converge-only). think(Converge) =>
      succeeds (uses call 3). think() again => Err(BudgetExhausted) (calls exhausted).
      Invariant: reasoning.txt s11 item 2. Note: max_calls=3 not 2 because must_converge
      fires at remaining<=1 — with max_calls=2 only 1 non-Converge call possible before
      forced convergence, masking overflow.
  T3. per_topic_budget_independence: budget("a",2), budget("b",3); exhaust "a"; assert
      "b" still has remaining_calls == 3.
  T4. invalid_transition_rejected: think(Observe), think(Verify) skipping Hypothesize +
      PlanAction => Err(InvalidTransition{from:Observe,to:Verify,allowed:[Hypothesize]}).
  T5. must_converge_at_budget_limit: budget(max_calls=3); 2 calls used; assert
      must_converge() == true; next think() must be Converge or rejected.
  T6. malformed_grounding_rejected: grounding=Some("no_colon_no_type") =>
      Err(MalformedGrounding).
  T7. status_step_free: budget(max_calls=2); think(Status); assert calls_used == 0.
      reasoning.txt s2 implies status is a free echo.
  T8. default_budget_applied_on_first_think: no set_budget; think(Observe) on new topic
      succeeds; default budget applied (max_calls=5, max_chars=800);
      get_budget(topic).unwrap().max_calls == 5.
  T9. first_step_must_be_observe: think(Hypothesize) on new topic =>
      Err(InvalidTransition) with from=None indication. Status/Converge bypass chain.
  T10. scope_enumeration_present_when_required: AgentGrcConfig require_scope_enumeration
       = true; think(Observe, scope=Some(vec!["ui","api","tests"])); succeeds. Then
       think(PlanAction) addressing only "ui","api"; assert ThinkResult.warnings contains
       WARN for unaddressed "tests". Invariant: reasoning.txt s10,
       technical-analysis-v3.txt Concern 3.
  T11. scope_enumeration_missing_when_required: require=true; think(Observe, scope=None)
       => Err or warning indicating scope required.
  T12. scope_enumeration_optional_when_not_required: require=false; think(Observe,
       scope=None) => succeeds, no warnings.
DETERMINISTIC_CHECK:
  - All 12 tests exist and FAIL (TDD)
    Command: cd .elai_cc && cargo test -p reasoning --no-run 2>&1 | grep -q "error"
  - Test names match pattern:
    Command: grep -c "#\[test\]" .elai_cc/crates/reasoning/tests/*.rs
    (should be >= 12)
ADVERSARIAL:
  - If tests pass against stubs, stubs too permissive. Stubs must use todo!() or
    unimplemented!(), not return placeholder Ok values.
  - If budget test uses sleep or timing, non-deterministic. Test: grep for
    "sleep\|Duration\|Instant" returns 0 matches.
ACCEPTANCE:
  - >= 12 test functions exist in reasoning/tests/
  - All tests fail to compile or fail at runtime (TDD contract)
  - Zero timing-dependent tests (grep confirms)
  - T10-T12 (scope enumeration) tests present and fail against stubs

----------------------------------------------------------------
### 2A.4 IMPLEMENT: GRC state machine logic
----------------------------------------------------------------
PRE_GATE:
  - Step 2A.3 complete (tests written, all currently fail)
  - No other session owns reasoning/ files (mcp_session_claim at EXECUTING entry)
TASK: Implement BasicGrcEngine in reasoning/src/engine.rs:
  I1. State transition validation: maintain last_state per topic. On think(), check
      step.step_type in last_state.allowed_next(). If not => Err(InvalidTransition).
      Special cases: first step in a topic must be Observe (no last_state); Status step
      allowed from any state, does not update last_state; Converge step allowed from any
      state (forced convergence path).
  I2. Grounding validation: For Observe, Hypothesize, Verify: if grounding None =>
      Err(UngroundedStep); if Some(s) parse as comma-separated "type:reference" pairs.
      Valid types: "file","grep","test","search","tool","training". Malformed =>
      Err(MalformedGrounding). Classify GroundingLevel: "test"/"search" -> Verified,
      "file"/"grep"/"tool" -> High, "training" -> Uncertain. Rationale: "search" =
      web-verified (s4: "searched" = VERIFIED). "tool" = tool output read this session
      (equivalent to file/grep = HIGH). Log warning for Uncertain (do not reject in 2A).
      For PlanAction, Act: grounding optional. For Status: no grounding required, no
      budget consumed.
  I3. Budget enforcement (CHECK-then-INCREMENT pattern):
      On each think() (except Status):
        - CHECK FIRST: if calls_used >= max_calls => Err(BudgetExhausted) immediately.
        - CHECK FIRST: if chars_used + step.content.len() > max_chars =>
          Err(BudgetExhausted) immediately.
        - CHECK FIRST: if must_converge() (remaining_calls <= 1) and step_type !=
          Converge => Err(BudgetExhausted) with message indicating forced convergence
          needed.
        - THEN INCREMENT: calls_used += 1, chars_used += step.content.len()
          (saturating_add to prevent u32 overflow).
      Rationale: increment-before-check corrupts budget state on the error path.
  I4. Topic isolation: each topic independent TopicState { budget, last_state, steps,
      grounding_log }. Default budget from global config (configurable via set_budget).
      reset_topic() clears all state for a topic.
  I5. ThinkResult construction: ThinkResult { step, budget_remaining, grounding_level,
      warnings }.
DETERMINISTIC_CHECK:
  - All 12 tests pass
    Command: cd .elai_cc && cargo test -p reasoning -- --nocapture 2>&1 | grep -E "test result:" | grep -q "0 failed"
  - No todo!() or unimplemented!() in engine.rs
    Command: grep -c "todo!\|unimplemented!" .elai_cc/crates/reasoning/src/engine.rs | grep -q "^0$"
  - No unwrap() in engine.rs
    Command: grep -c "\.unwrap()" .elai_cc/crates/reasoning/src/engine.rs | grep -q "^0$"
ADVERSARIAL:
  - If allowed_next() returns empty for Verify (terminal state), valid chains cannot
    complete. Test: Verify.allowed_next() must contain at least Observe (new chain
    start) or be explicitly terminal with documented reason.
  - If chars_used overflows u32 at ~4GB, budget tracking breaks. Test: chars_used
    checked before increment (saturating_add or checked_add).
  - If topic HashMap grows unbounded, memory leaks. Test: after reset_topic(), topic
    entry fully removed from map (not just zeroed).
ACCEPTANCE:
  - cargo test -p reasoning: 0 failures, >= 12 tests pass
  - cargo check --workspace: 0 errors
  - grep todo!/unimplemented!/unwrap() in engine.rs: 0 matches

----------------------------------------------------------------
### 2A.5 MEASURE: performance and correctness metrics
----------------------------------------------------------------
PRE_GATE: Step 2A.4 complete (all tests pass)
TASK:
  - Measure per-step validation overhead (target: <5ms for pure logic, no I/O)
  - Measure memory per topic (target: <1KB base + step history)
  - Record: test count, LOC, compile time delta from Phase 0 baseline
  - Write metrics to .elai_cc/metrics/phase_2A.toml
DETERMINISTIC_CHECK:
  - Metrics file exists and contains required fields
    Command: test -f .elai_cc/metrics/phase_2A.toml && grep -q "validation_overhead_us" .elai_cc/metrics/phase_2A.toml
  - Validation overhead < 5ms (5000us)
    Command: grep "validation_overhead_us" .elai_cc/metrics/phase_2A.toml (value < 5000)
ADVERSARIAL:
  - If measurement uses wall-clock time in CI, non-deterministic. Use std::hint::black_box
    + std::time::Instant for local benchmarks only (not CI gate). Report median of 1000
    iterations. Do NOT add criterion without explicit user approval. Note: Instant for
    benchmarks acceptable under entropy quarantine — measurement metadata, not output.
ACCEPTANCE:
  - .elai_cc/metrics/phase_2A.toml exists with all required fields
  - validation_overhead_us < 5000
  - cargo test -p reasoning: 0 failures

EXIT CRITERIA (Phase 2A)
  GRC enforces grounding on every step that requires it (Observe, Hypothesize, Verify).
  Budget overflow triggers convergence or returns BudgetExhausted error.
  Invalid state transitions rejected with InvalidTransition error.
  Malformed grounding strings rejected with MalformedGrounding error.
  Status steps free (no budget consumed, no grounding required).
  Per-topic budgets independent.
  Scope enumeration validated when require_scope_enumeration=true (WARN on unaddressed).
  All 12+ tests pass. Zero todo!/unimplemented!/unwrap() in engine code.

GROUNDING SUMMARY (Phase 2A)
  "GRC state machine: observe->hypothesize->plan_action->act->verify" -> GROUNDED:
    reasoning.txt section 2
  "Budget per-topic: max_calls 1-12, max_chars 200-1500" -> GROUNDED: reasoning.txt s3
  "Grounding required on every step" -> GROUNDED: reasoning.txt s2, s11 item 1
  "Budget overflow forces convergence" -> GROUNDED: reasoning.txt s3, s11 item 2
  "Status step is free echo" -> VERIFIED: current workflow uses
    mcp_think(step_type="status") (CLAUDE.md GRC tools section)
  "<5ms validation overhead" -> UNGROUNDED: assumed reasonable; verified empirically in 2A.5
  "Forced convergence at remaining_calls<=1" -> GROUNDED: BudgetThinker (OpenReview 2025)
  "Claim decomposition in grounding validation" -> GROUNDED: RARR (arxiv:2210.08726),
    SAFE (arxiv:2403.18802)
  "Hard/soft grounding: reject UngroundedStep but warn on Uncertain" -> GROUNDED: DSPy
    Assertions (arxiv:2312.13382)
  "State transition validation via allowed_transitions" -> GROUNDED: standard FSM pattern;
    guidance-ai uses FSM for token masking
  "thiserror for GrcError" -> VERIFIED: thiserror 2.x standard Rust error derive
  "Memory per topic <1KB base" -> UNGROUNDED: assumed from struct sizes; verified in 2A.5
  "ThinkStep.scope: Option<Vec<String>> for scope enumeration" -> GROUNDED:
    reasoning.txt s10, technical-analysis-v3.txt Concern 3 Mechanism 1
  "Scope enumeration validated when require_scope_enumeration=true" -> GROUNDED:
    technical-analysis-v3.txt testable invariants

================================================================
## PHASE 2B: GROUNDING VERIFICATION
================================================================
Prerequisites: Phase 2A complete (all 9+ GRC tests pass, cargo test -p reasoning 0 failures)
SCOPE: see docs/specs/guide/implementation-guide.txt

GOAL
Implement verification methods that check claims against real sources before execution.
Phase 2A validates grounding format (string parsing). Phase 2B resolves grounding claims by
actually checking files, source code, tests, web, and APIs. Ungrounded claims in a plan
block the PLANNING -> EXECUTING FSM transition. Six verification methods: FILE_CHECK,
SOURCE_CHECK, TEST_CHECK, WEB_CHECK, API_CHECK, API_REGISTRY_CHECK. Each returns binary
pass/fail with evidence string. No confidence scores — checks are deterministic. Plan
grounding sweep: before EXECUTING, GRC scans all claims in a plan, classifies each as
GROUNDED / VERIFY_NEEDED / UNGROUNDED, runs verification on VERIFY_NEEDED, and blocks
execution if any remain UNGROUNDED after resolution. Phase 2B also resolves UNCERTAIN
grounding levels from phase 2A. Steps with GroundingLevel::Uncertain that can be resolved
via FILE_CHECK or SOURCE_CHECK are auto-promoted to Verified or High. Those that cannot be
resolved remain UNCERTAIN and are flagged (per-agent config in 2C handles auto-rejection).
Forward dependency: Phase 4A changes verification execution model — agents proxy
verification via IPC GroundingRequest messages through the orchestrator. Phase 2B
implements verification as direct function calls; 4A wraps them behind IPC.

READ BEFORE IMPLEMENTING
  docs/specs/crates/reasoning.txt             Section 4 (grounding verification methods)
  docs/specs/crates/reasoning.txt             Section 5 (fact-based planning protocol)
  docs/specs/crates/reasoning.txt             Section 8 (inline feasibility check — phase 2D)
  docs/specs/crates/reasoning.txt             Section 9 (process isolation — forward dep on 4A)
  docs/specs/crates/reasoning.txt             Section 10 (KEY TYPES: GroundingResult, VerificationResult)
  docs/specs/crates/reasoning.txt             Section 11 (TESTABLE INVARIANTS: item tagged [2B])
  docs/specs/architecture-overview.txt        Section 4 (GrcEngine trait: verify_claim, ground_plan)
  docs/specs/guide/steps/phase-2A.txt         Step 2A.1 (stub definitions this phase replaces)
  docs/specs/crates/tools.txt                 (FILE_CHECK, SOURCE_CHECK use tools/ crate)

DESIGN DECISIONS (grounded)
  D1. Binary verification, not scalar confidence. All 6 methods return VerificationResult
      { passes: bool, evidence: String }. Grounding: MiniCheck (Tang et al., EMNLP 2024,
      arxiv:2404.10774); Google Vertex AI grounding API uses binary classification.
  D2. Claim decomposition for plan sweep only. Structural decomposition (extract file
      paths, function names, API references), not semantic (no NLI model). Grounding:
      RARR (arxiv:2210.08726), SAFE (Wei et al., 2024). Phase 2A already validates
      grounding format as "type:reference" pairs (2A.4 I2).
  D3. VerificationMethod enum for dispatch. New enum: VerificationMethod { FileCheck,
      SourceCheck, TestCheck, WebCheck, ApiCheck, ApiRegistryCheck }. Allows per-method
      availability filtering (Phase 4A IPC proxy). Grounding: reasoning.txt s9 (L140-147).
  D4. verified_at via entropy-quarantined timestamp. GroundingResult includes verified_at:
      Timestamp via core::entropy::TimeSource trait. Deterministic testing.
      Grounding: implementation-guide.txt L46; reasoning.txt L184.
  D5. tools/ crate dependency for FILE_CHECK and SOURCE_CHECK. FILE_CHECK uses
      tools::read_file. SOURCE_CHECK uses tools::read_file + parsing. First reasoning/ ->
      tools/ coupling. Grounding: crate map (implementation-guide.txt L64). Verified
      knowledge/ does NOT depend on reasoning/.
  D6. GrcEngine trait signature update (BREAKING CHANGE from 2A stubs).
      2A: fn ground_plan(&self, plan: &str) -> Vec<GroundingResult>
      2B: fn ground_plan(&self, plan: &str) -> PlanGroundingResult
      PRE_CHECK: grep phase-2A tests for "ground_plan".
  D7. verify_claim gets a context parameter. 2A: 2 args. 2B:
      fn verify_claim(&self, claim: &str, source: &str, context: &str) ->
      Result<VerificationResult, GrcError>. Context needed for SOURCE_CHECK
      (claim=symbol, context=file_path) and TEST_CHECK (claim=test_name,
      context=crate_name).
  D8. TestRunner trait abstraction for TEST_CHECK. Production: CargoTestRunner calls
      BashSandbox::execute("cargo test --exact {name} -p {crate}", sandbox_config) with
      30s timeout. Test: MockTestRunner. Grounding: tools.txt s9 L184-186.

RISKS TO WATCH
  R-2B-1: tools/ dependency introduces coupling earlier. Mitigation: reasoning/ uses only
    tools::read_file (leaf function).
  R-2B-2: WEB_CHECK and API_CHECK require network. Phase 2B tests must NOT require
    network. Mitigation: mock/stub. Real network only in #[cfg(feature="network")] tests.
  R-2B-3: Plan sweep must handle empty plan — empty plan = trivially GROUNDED.
  R-2B-4: Phase 4A wraps verification behind IPC. verify_claim takes &self (not
    &mut self), stateless per call.

----------------------------------------------------------------
### 2B.1 SKELETON: verification types + trait extension
----------------------------------------------------------------
PRE_GATE:
  - Phase 2A tests all pass
    Command: cd .elai_cc && cargo test -p reasoning 2>&1 | grep "test result:" | grep -q "0 failed"
  - verify_claim and ground_plan stubs exist (placed by 2A.1)
    Command: grep -q "fn verify_claim" .elai_cc/crates/reasoning/src/engine.rs && grep -q "fn ground_plan" .elai_cc/crates/reasoning/src/engine.rs
  - VerificationResult and GroundingResult stub types exist
    Command: grep -q "VerificationResult" .elai_cc/crates/reasoning/src/types.rs && grep -q "GroundingResult" .elai_cc/crates/reasoning/src/types.rs
TASK:
  - Add VerificationMethod enum to reasoning/src/types.rs:
      VerificationMethod { FileCheck, SourceCheck, TestCheck, WebCheck, ApiCheck, ApiRegistryCheck }
      #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
      fn requires_network(&self) -> bool (WebCheck, ApiCheck, ApiRegistryCheck = true)
      fn requires_tools(&self) -> bool (FileCheck, SourceCheck, TestCheck = true)
  - PRE-CHECK: grep phase 2A tests for GroundingResult construction. Replace any
    method: String with VerificationMethod enum.
  - Update GroundingResult: GroundingResult { claim: String, method: VerificationMethod,
    level: GroundingLevel, source: String, verified_at: u64 }
    verified_at = u64 unix timestamp from core::entropy::TimeSource.
  - Update VerificationResult: { claim: String, passes: bool, evidence: String,
    method_used: VerificationMethod }
  - Add PlanGroundingResult: { claims: Vec<GroundingResult>, all_grounded: bool,
    ungrounded: Vec<String>, verify_needed: Vec<String> }
    Semantics: verify_needed contains claims of type "training:" that COULD NOT be
    auto-resolved. Auto-resolved "training:" claims are in claims with level=Verified,
    NOT in verify_needed. verify_needed is SUBSET of ungrounded.
    all_grounded = ungrounded.is_empty() (verify_needed does not affect this).
  - Add GrcError::VerificationError { method: VerificationMethod, claim: String,
    reason: String }
  - Add Verifier trait to reasoning/src/verification.rs:
      trait Verifier: Send + Sync {
        fn verify(&self, method: VerificationMethod, claim: &str, context: &str)
          -> Result<VerificationResult, GrcError>;
        fn supported_methods(&self) -> &[VerificationMethod];
      }
  - Add TestRunner trait to reasoning/src/verification.rs:
      trait TestRunner: Send + Sync {
        fn run_test(&self, name: &str, crate_name: &str) -> TestRunResult;
      }
      TestRunResult { passed: bool, output: String, duration_ms: u64 }
      Implementations (stubs): CargoTestRunner { sandbox: Box<dyn BashSandbox>,
      timeout_ms: u64 }; MockTestRunner { results: HashMap<String, TestRunResult> }
  - Update GrcEngine trait stubs (BREAKING CHANGES from 2A — see D6, D7):
      fn verify_claim(&self, claim: &str, source: &str, context: &str)
        -> Result<VerificationResult, GrcError>  (3 args now)
      fn ground_plan(&self, plan: &str) -> PlanGroundingResult  (new return type)
  - Add tools/ dependency to reasoning/Cargo.toml.
  - Stub implementations: LocalVerifier (FILE_CHECK, SOURCE_CHECK, TEST_CHECK),
    NetworkVerifier (WEB_CHECK, API_CHECK, API_REGISTRY_CHECK) — both return todo!()
    LocalVerifier::new(verifier: Box<dyn TestRunner>) — takes TestRunner for TEST_CHECK
  - Re-export new types from reasoning/src/lib.rs
DETERMINISTIC_CHECK:
  - cargo check on workspace succeeds
    Command: cd .elai_cc && cargo check 2>&1 | grep -c "error\[" | grep -q "^0$"
  - New types exist:
    Command: grep -c "pub enum VerificationMethod" .elai_cc/crates/reasoning/src/types.rs | grep -q "1"
    Command: grep -c "pub struct PlanGroundingResult" .elai_cc/crates/reasoning/src/types.rs | grep -q "1"
    Command: grep -c "pub trait Verifier" .elai_cc/crates/reasoning/src/verification.rs | grep -q "1"
  - VerificationMethod has exactly 6 variants:
    Command: grep -A7 "pub enum VerificationMethod" .elai_cc/crates/reasoning/src/types.rs | grep -c "Check" | grep -q "6"
  - GroundingResult has verified_at field:
    Command: grep -A6 "pub struct GroundingResult" .elai_cc/crates/reasoning/src/types.rs | grep -q "verified_at"
  - tools/ in reasoning/Cargo.toml deps:
    Command: grep -q "tools" .elai_cc/crates/reasoning/Cargo.toml
  - Phase 2A tests still pass (no regression):
    Command: cd .elai_cc && cargo test -p reasoning 2>&1 | grep "test result:" | grep -q "0 failed"
ADVERSARIAL:
  - If GroundingResult.method is String, dispatch fragile. Test: grep "method: String"
    in GroundingResult must be 0.
  - If Verifier trait not object-safe, cannot use dyn Verifier. Test: compile
    let _: Box<dyn Verifier>;
  - If tools/ dep creates circular dependency, won't compile. Test: cargo check.
ACCEPTANCE:
  - cargo check succeeds (0 errors)
  - VerificationMethod enum has 6 variants
  - Verifier trait is object-safe (Box<dyn Verifier> compiles)
  - Phase 2A tests still pass (0 regressions)
GROUNDING:
  - "6 verification methods" -> GROUNDED: reasoning.txt section 4 (L46-55)
  - "verified_at in GroundingResult" -> GROUNDED: reasoning.txt L184
  - "tools/ dependency for reasoning/" -> GROUNDED: implementation-guide.txt L64
  - "Verifier trait with Send + Sync" -> VERIFIED (inference): phase 4A IPC

----------------------------------------------------------------
### 2B.2 VERIFY: compilation + regression check
----------------------------------------------------------------
PRE_GATE:
  - Step 2B.1 complete (new types and trait exist, workspace compiles)
    Command: test -f .elai_cc/crates/reasoning/src/verification.rs && echo PASS
TASK:
  - Run cargo check on full workspace. Fix any compile errors.
  - Verify reasoning/ depends only on core/, tools/, thiserror, serde (no providers/ yet,
    no knowledge/). providers/ comes in 2C or stays out.
  - Verify no circular dep: tools/ must NOT depend on reasoning/.
DETERMINISTIC_CHECK:
  - cargo check returns 0 errors
    Command: cd .elai_cc && cargo check 2>&1 | grep "error\[" | wc -l | grep -q "^0$"
  - No circular dep:
    Command: grep -q "reasoning" .elai_cc/crates/tools/Cargo.toml && echo "FAIL: circular dep" || echo "PASS"
  - reasoning/ deps are only core, tools, thiserror, serde:
    Command: grep "^\[dependencies" .elai_cc/crates/reasoning/Cargo.toml -A 30 | grep -v "core\|tools\|thiserror\|serde\|^\[\|^$\|^#" | grep -v "^$" | wc -l
ADVERSARIAL:
  - If reasoning/ pulls in knowledge/ via tools/ transitive dep, compile times grow.
    Test: cargo tree -p reasoning | grep knowledge must return empty.
ACCEPTANCE:
  - cargo check exit code 0
  - reasoning/ depends only on core/, tools/, thiserror, serde
  - No circular dependency

----------------------------------------------------------------
### 2B.3 TEST: write failing tests (TDD)
----------------------------------------------------------------
PRE_GATE:
  - Step 2B.2 passes (workspace compiles clean)
  - Verifier trait exists in verification.rs
    Command: grep -q "pub trait Verifier" .elai_cc/crates/reasoning/src/verification.rs
TASK: Write 16 tests in reasoning/tests/verification_tests.rs:
  T1. file_check_existing_file: temp file. verify(FileCheck, "path/to/temp", "") =>
      passes==true, evidence contains path. Invariant: reasoning.txt L46.
  T2. file_check_nonexistent_file: verify(FileCheck, "nonexistent_file_xyz.rs", "") =>
      passes==false.
  T3. source_check_function_exists: temp file containing "fn hello_world()".
      verify(SourceCheck, "hello_world", path) => passes==true, evidence contains
      "hello_world".
  T4. source_check_function_missing: temp file no matching function.
      verify(SourceCheck, "nonexistent_fn", path) => passes==false.
  T5. test_check_passing_test: verify(TestCheck, "test_name", "test_module_path").
      Note: requires mock test runner or pre-existing passing test.
  T6. web_check_stub_returns_error: verify(WebCheck, "some algorithm", "") in unit test
      (no network) => GrcError::VerificationError or stub indicating network unavailable.
      Invariant: R-2B-2.
  T7. api_check_stub_returns_error: same as T6 but for ApiCheck.
  T8. api_registry_check_stub_returns_error: same as T6 for ApiRegistryCheck.
  T9. plan_grounding_sweep_blocks_ungrounded: plan with claims ["file:src/main.rs"
      (exists), "file:nonexistent.rs" (missing)]. ground_plan(plan) => all_grounded==
      false; ungrounded contains "file:nonexistent.rs". Invariant: reasoning.txt L199.
  T10. plan_grounding_sweep_passes_all_grounded: plan with only valid file claims =>
       all_grounded==true, ungrounded empty.
  T11. plan_grounding_sweep_empty_plan: ground_plan("") => all_grounded==true (trivially).
       Invariant: R-2B-3.
  T12. uncertain_claim_auto_resolution: ThinkStep grounding "training:some_claim"
       (UNCERTAIN). Claim references file that exists. After verification level promoted
       to Verified.
  T13. verification_method_requires_network: WebCheck.requires_network()==true;
       FileCheck.requires_network()==false.
  T14. verifier_trait_object_safety: let _: Box<dyn Verifier> = Box::new(LocalVerifier::
       new(...)); must compile. LocalVerifier::new takes Box<dyn TestRunner> — pass
       MockTestRunner.
  T15. plan_sweep_adversarial_regex: plan text "// see file:fake_path for details\n
       Grounding: file:src/main.rs". ground_plan extracts only real claim
       (file:src/main.rs), not embedded "file:fake_path".
  T16. verify_claim_strips_prefix: verify_claim("file:src/main.rs", "", "") => Verifier
       receives "src/main.rs" (no "file:" prefix). Validates BLOCK fix in I3.
DETERMINISTIC_CHECK:
  - All 16 tests exist
    Command: grep -c "#\[test\]" .elai_cc/crates/reasoning/tests/verification_tests.rs
    (should be >= 16)
  - Tests compile:
    Command: cd .elai_cc && cargo test -p reasoning --test verification_tests --no-run 2>&1; [ $? -eq 0 ] && echo "COMPILE OK"
  - Tests fail at RUNTIME (TDD: stubs panic via todo!()):
    Command: cd .elai_cc && cargo test -p reasoning --test verification_tests 2>&1 | grep -q "FAILED"
  - No network calls in tests:
    Command: grep -c "reqwest\|hyper\|TcpStream\|connect\|http://" .elai_cc/crates/reasoning/tests/verification_tests.rs | grep -q "^0$"
  - No timing-dependent tests:
    Command: grep -c "sleep\|Duration\|Instant\|SystemTime" .elai_cc/crates/reasoning/tests/verification_tests.rs | grep -q "^0$"
ADVERSARIAL:
  - If T9 passes against stubs, ground_plan stub too permissive (returning empty vec).
    Test: stub must use todo!() or unimplemented!().
  - If T12 uses real FS without temp dir cleanup, tests leak files. Test: tempfile or
    std::env::temp_dir, cleaned in drop.
  - If T5 spawns actual cargo test, runtime explodes. Test: T5 must use a mock test
    runner.
ACCEPTANCE:
  - >= 16 test functions exist
  - All tests fail against stubs (TDD contract)
  - Zero network calls in test code
  - Zero timing-dependent code in tests
  - All temp files use tempfile crate

----------------------------------------------------------------
### 2B.4 IMPLEMENT: verification methods + plan sweep
----------------------------------------------------------------
PRE_GATE:
  - Step 2B.3 complete (tests written, all currently fail)
  - No other session owns reasoning/ files (mcp_session_claim at EXECUTING entry)
TASK:
  I1. LocalVerifier implementation (FILE_CHECK, SOURCE_CHECK, TEST_CHECK):
      LocalVerifier takes Box<dyn TestRunner> at construction (DI for D8).
      FILE_CHECK: tools::read_file(path, None, None). Claim format: file path ("file:"
        prefix stripped before passing to read_file). FileReadResult => passes=true,
        evidence="File exists: {path}, {total_lines} lines". NotFound => passes=false,
        evidence="File not found: {path}". Grounding: tools.txt s2 (L23).
      SOURCE_CHECK: tools::read_file + string search. Claim=symbol_name. Context=file_path.
        Search content for regex r"(fn|struct|enum|trait|impl|const|static|type|mod)\s+
        {symbol_name}\b". Found => passes=true, evidence="Found {kind} {name} at
        {file}:{line}". Not => passes=false, evidence="Symbol {name} not found in {file}".
      TEST_CHECK: delegate to injected TestRunner (D8). Claim=test_name. Context=crate_name.
        Production: CargoTestRunner calls BashSandbox::execute("cargo test --exact {name}
        -p {crate}", sandbox_config) with 30s timeout. Grounding: tools.txt s9 L184-186.
        exit_code==0 => passes=true, evidence="Test {name} passed". Non-zero => passes=
        false, evidence="Test {name} failed: {stderr}". Timeout => passes=false,
        evidence="Test {name} timed out after 30s".
  I2. NetworkVerifier stubs (WEB_CHECK, API_CHECK, API_REGISTRY_CHECK):
      Phase 2B implements trait + dispatch but NOT real network calls.
      WEB_CHECK: returns VerificationError("network verification not implemented in
      phase 2B"). API_CHECK, API_REGISTRY_CHECK: same stub. Real network deferred to
      provider integration / 4A IPC proxy.
  I3. verify_claim implementation on BasicGrcEngine:
      Replace unimplemented!() stub from phase 2A.
      Signature change (D7): fn verify_claim(&self, claim: &str, source: &str,
        context: &str) -> Result<VerificationResult, GrcError>
      Parse claim to determine VerificationMethod from grounding format "type:reference":
        "file:..." -> FileCheck (strip "file:", remainder = path)
        "grep:..." -> FileCheck (strip "grep:")
        "test:..." -> TestCheck (strip "test:", context = crate name)
        "search:..." -> WebCheck (strip "search:")
        "tool:..." -> SourceCheck (strip "tool:", context = file path)
        "training:..." -> no verification (flag UNCERTAIN, return passes=false)
      Dispatch to appropriate Verifier via self.verifier.verify(method, stripped_claim,
      context). BasicGrcEngine holds Box<dyn Verifier> (injected at construction for
      testability). NOTE: "type:" prefix ALWAYS stripped before passing to Verifier.
  I4. ground_plan implementation on BasicGrcEngine:
      Replace unimplemented!() stub from phase 2A.
      Signature change (D6): fn ground_plan(&self, plan: &str) -> PlanGroundingResult
      Algorithm:
        1. Parse plan text into claims using grounding format regex (pattern:
           "type:reference" where type in {file,grep,test,search,tool,training}).
        2. For each claim, call verify_claim().
        3. Classify: passes=true -> GROUNDED; passes=false -> UNGROUNDED. type "training"
           -> VERIFY_NEEDED (attempt FILE_CHECK/SOURCE_CHECK auto-resolution; if fails,
           UNGROUNDED).
        4. Construct PlanGroundingResult { claims, all_grounded, ungrounded, verify_needed }.
        5. Empty plan (0 claims) -> PlanGroundingResult { all_grounded: true, ... }.
      verified_at: from injected TimeSource (core::entropy).
  I5. UNCERTAIN claim auto-resolution:
      When GRC encounters step with GroundingLevel::Uncertain:
        - Extract claim reference from grounding string.
        - Attempt FILE_CHECK (path) or SOURCE_CHECK (symbol name).
        - If passes: promote to Verified.
        - If fails or not file/source-resolvable: keep Uncertain.
      Called during think() for steps with Uncertain grounding (enhances 2A logic).
DETERMINISTIC_CHECK:
  - All 16 tests pass
    Command: cd .elai_cc && cargo test -p reasoning -- --nocapture 2>&1 | grep "test result:" | grep -q "0 failed"
  - Phase 2A tests still pass (regression check)
  - No todo!() or unimplemented!() in verification.rs or engine.rs verify_claim/ground_plan
    Command: grep -c "todo!\|unimplemented!" .elai_cc/crates/reasoning/src/verification.rs | grep -q "^0$"
    Command: grep -A30 "fn verify_claim" .elai_cc/crates/reasoning/src/engine.rs | grep -c "todo!\|unimplemented!" | grep -q "^0$"
    Command: grep -A30 "fn ground_plan" .elai_cc/crates/reasoning/src/engine.rs | grep -c "todo!\|unimplemented!" | grep -q "^0$"
  - No unwrap() in verification.rs
    Command: grep -c "\.unwrap()" .elai_cc/crates/reasoning/src/verification.rs | grep -q "^0$"
  - Network stubs return VerificationError (not panic):
    Command: grep -c "todo!\|unimplemented!\|panic!" .elai_cc/crates/reasoning/src/verification.rs | grep -q "^0$"
ADVERSARIAL:
  - If verify_claim parses "file:src/main.rs" but passes prefix-included to FILE_CHECK,
    looks for file literally named "file:src/main.rs". Test: T1/T2 use actual paths.
  - If ground_plan regex too greedy, may match "file:" inside comments. Test: adversarial
    plan with "the word file:fake in a comment".
  - If TEST_CHECK no timeout, hanging test blocks indefinitely. Test: T5 mock verifies
    timeout set.
  - If Box<dyn Verifier> not injected, BasicGrcEngine cannot be tested with mocks. Test:
    constructor takes verifier parameter.
ACCEPTANCE:
  - All 16+ tests pass, 0 failures
  - All phase 2A tests still pass (0 regressions)
  - No todo!/unimplemented!/unwrap()/panic! in verification.rs
  - verify_claim and ground_plan are no longer stubs
  - Network stubs return typed errors (not panics)

----------------------------------------------------------------
### 2B.5 MEASURE: verification performance
----------------------------------------------------------------
PRE_GATE: Step 2B.4 complete (all tests pass)
TASK:
  - FILE_CHECK latency target: <2ms for local file existence check
  - SOURCE_CHECK latency target: <10ms for reading + searching a 1000-line file
  - ground_plan latency for plan with 20 claims target: <100ms total
  - Record: test count, LOC, compile time delta from Phase 2A baseline
  - Record: number of verification methods implemented vs stubbed (should be 3/6)
  - Write metrics to .elai_cc/metrics/phase_2B.toml
DETERMINISTIC_CHECK:
  - Metrics file exists with required fields
    Command: test -f .elai_cc/metrics/phase_2B.toml && grep -q "file_check_latency_us" .elai_cc/metrics/phase_2B.toml
    Command: grep -q "source_check_latency_us" .elai_cc/metrics/phase_2B.toml
    Command: grep -q "plan_sweep_latency_us" .elai_cc/metrics/phase_2B.toml
    Command: grep -q "methods_implemented" .elai_cc/metrics/phase_2B.toml
  - FILE_CHECK < 2000us
    Command: val=$(grep "file_check_latency_us" .elai_cc/metrics/phase_2B.toml | grep -oP '\d+'); [ "$val" -lt 2000 ] && echo PASS
  - SOURCE_CHECK < 10000us
    Command: val=$(grep "source_check_latency_us" .elai_cc/metrics/phase_2B.toml | grep -oP '\d+'); [ "$val" -lt 10000 ] && echo PASS
  - plan_sweep < 100000us
    Command: val=$(grep "plan_sweep_latency_us" .elai_cc/metrics/phase_2B.toml | grep -oP '\d+'); [ "$val" -lt 100000 ] && echo PASS
ADVERSARIAL:
  - SOURCE_CHECK benchmark on 10-line file does not reflect real usage. Test: must use
    >= 1000 lines.
  - plan_sweep benchmark with 1 claim does not test scaling. Test: 20 claims.
ACCEPTANCE:
  - .elai_cc/metrics/phase_2B.toml exists with all required fields
  - FILE_CHECK latency < 2000us
  - SOURCE_CHECK latency < 10000us (for 1000-line file)
  - plan_sweep latency < 100000us (for 20-claim plan)
  - cargo test -p reasoning: 0 failures

EXIT CRITERIA (Phase 2B)
  All 6 verification methods have type-level representation (VerificationMethod enum).
  3 methods fully implemented (FILE_CHECK, SOURCE_CHECK, TEST_CHECK).
  3 methods stubbed with typed errors (WEB_CHECK, API_CHECK, API_REGISTRY_CHECK).
  verify_claim dispatches to correct verifier based on grounding format.
  ground_plan decomposes plan into claims, verifies each, returns PlanGroundingResult.
  Ungrounded claims detected and reported (all_grounded == false when any claim fails).
  UNCERTAIN grounding auto-resolved via FILE_CHECK/SOURCE_CHECK where possible.
  Empty plan handled (trivially grounded, no panic).
  All 16+ tests pass. All phase 2A tests still pass.
  Zero todo!/unimplemented!/unwrap()/panic! in verification code.
  No network calls in any test.

GROUNDING SUMMARY (Phase 2B)
  "6 verification methods..." -> GROUNDED: reasoning.txt s4 (L46-55)
  "Binary pass/fail verification" -> GROUNDED: MiniCheck (EMNLP 2024, arxiv:2404.10774);
    Google Vertex AI grounding fast mode
  "Structural claim decomposition (regex, not NLI)" -> GROUNDED: RARR (arxiv:2210.08726);
    typed "type:reference" format from phase 2A
  "Plan grounding sweep blocks EXECUTING" -> GROUNDED: reasoning.txt s4 L64-65, s11 L199
  "GroundingResult includes verified_at" -> GROUNDED: reasoning.txt L184
  "tools/ as reasoning/ dependency" -> GROUNDED: implementation-guide.txt L64
  "No circular dep (tools -> knowledge, knowledge !-> reasoning)" -> VERIFIED:
    implementation-guide.txt L65, L64
  "Verifier trait must be Send + Sync" -> VERIFIED (inference): phase 4A process
    isolation with IPC (reasoning.txt s9 L134); L137 GroundingRequest message
  "FILE_CHECK <2ms, SOURCE_CHECK <10ms targets" -> UNGROUNDED: assumed reasonable
  "3/6 methods implemented in 2B, 3 network-dependent stubbed" -> GROUNDED:
    reasoning.txt s9 L134 (agents have no network access except researcher); network
    methods need orchestrator proxy (4A)

================================================================
## PHASE 2C: PER-AGENT GRC CONFIGURATION
================================================================
Prerequisites: Phase 2B complete (grounding verification methods implemented, plan sweep
blocks ungrounded claims)
SCOPE: see docs/specs/guide/implementation-guide.txt

GOAL
Different agent roles get different GRC budgets, step sequences, grounding constraints, and
allowed verification methods. The GRC engine routes configuration by role at spawn time.
Auditor gets double-verify with VERIFIED-only grounding. Context-compiler gets observe-only
with FILE_CHECK/SOURCE_CHECK only. Config is layered: global defaults in elai_cc.toml,
per-role overrides in [agents.{role}.grc] tables, validated at load time.

READ BEFORE IMPLEMENTING
  docs/specs/crates/reasoning.txt             Section 7 (per-agent config: 5 role definitions)
  docs/specs/crates/reasoning.txt             Section 9 (GRC under process isolation: IPC proxy)
  docs/specs/crates/reasoning.txt             Section 10 (KEY TYPES: AgentGrcConfig)
  docs/specs/crates/reasoning.txt             Section 11 (TESTABLE INVARIANTS: item 9 is phase-2C)
  docs/specs/architecture-overview.txt        Section 9 (agent roles and model assignments)
  docs/specs/crates/orchestrator.txt          Section 7 (agent supervision, checkpoint, restart)
  docs/specs/crates/orchestrator.txt          Lines 440-444 (budget enforcement in orchestrator)
  docs/specs/integration/prompt-architecture.txt  Lines 61-86 ([agents.{role}] TOML pattern)

DESIGN DECISIONS (grounded)
  D1. Per-agent transition maps extending GrcState::allowed_next(). Auditor requires
      [observe, hypothesize, verify, verify] — double verify. Phase 2A's allowed_next()
      does NOT permit verify->verify. Solution: AgentGrcConfig carries optional
      transition_overrides: HashMap<GrcState, Vec<GrcState>> merged with default. Auditor
      override: {Verify: [Verify, Observe]}. ADDITIVE-only (cannot remove default
      transitions). Grounding: reasoning.txt s7 L97.
  D2. Config validation at load time (fail-fast). Validation rules: (a) required_steps
      non-empty, (b) required_steps forms valid chain through default + overridden
      transitions, (c) budget.max_calls >= required_steps.len(), (d) csq_required implies
      at least one Verify in required_steps, (e) allowed_methods subset of all known
      VerificationMethod variants. Grounding: standard fail-fast.
  D3. TOML layering: global defaults + per-role overrides. [grc.defaults] table.
      [agents.{role}.grc] per-role overrides. Field-by-field merge (not deep). Pattern
      matches existing [agents.{role}.supervision], [agents.{role}.slo_tier],
      [agents.{role}.sandbox], [agents.{role}.role_prompt].
      NOTE: existing TOML keys use UNDERSCORES ([agents.context_compiler],
      [agents.prompt_engineer]) per prompt-architecture.txt L74-82. Phase 2C MUST follow:
      [agents.context_compiler.grc], NOT [agents.context-compiler.grc].
  D4. Allowed verification methods per role (POLICY layer). AgentGrcConfig gains
      allowed_methods: Vec<VerificationMethod>. POLICY restriction: limits which methods
      agent permitted to request, independent of infrastructure CAPABILITY. Distinction
      from reasoning.txt s9 L140-146: section 9 = CAPABILITY (what orchestrator CAN
      provide). allowed_methods = POLICY (what role's GRC config PERMITS). Policy <=
      Capability. Context-compiler policy: [FILE_CHECK, SOURCE_CHECK] only (s7 L105-106).
      Executor/auditor: all methods. Phase 2C enforces in-process; phase 4A adds second
      enforcement on GroundingRequest IPC messages.
      SPEC UPDATE NEEDED: reasoning.txt L186 AgentGrcConfig should add allowed_methods
      and transition_overrides. (Cannot edit reasoning.txt — owned by session s-1003192.
      Flag for reconciliation.)
  D5. Anti-rubber-stamp enforcement for auditor. "Must find >= 1 issue per review"
      (reasoning.txt L99). Enforced in GRC: when auditor reaches Converge, engine checks
      at least one ThinkStep history has step_type=Verify with finding (content contains
      "FINDING:" prefix). If no finding: Converge rejected with new variant
      InsufficientFindings. Grounding: reasoning.txt s7 L99.

----------------------------------------------------------------
### 2C.1 SKELETON: AgentGrcConfig type + config loading stubs
----------------------------------------------------------------
PRE_GATE:
  - Phase 2B tests pass (grounding verification works)
    Command: cd .elai_cc && cargo test -p reasoning 2>&1 | grep -E "test result:" | grep -q "0 failed"
  - GrcEngine trait exists with think/set_budget/get_budget/reset_topic
    Command: grep -q "fn think" .elai_cc/crates/reasoning/src/engine.rs
  - VerificationMethod enum exists (from phase 2B)
    Command: grep -q "enum VerificationMethod\|enum GroundingMethod" .elai_cc/crates/reasoning/src/types.rs
TASK:
  - Define AgentGrcConfig in reasoning/src/types.rs:
      AgentGrcConfig {
        role: String,
        budget: GrcBudget,
        required_steps: Vec<GrcState>,
        grounding_level: GroundingLevel,   // minimum accepted level
        csq_required: bool,
        allowed_methods: Vec<VerificationMethod>,
        transition_overrides: HashMap<GrcState, Vec<GrcState>>,
        require_scope_enumeration: bool,   // default: true for executor, false for memory_writer
      }
      require_scope_enumeration: when true, Observe step's ThinkStep.scope must be
      populated. Orchestrator validates every scope item appears in subsequent
      plan_action. Unaddressed -> WARN. Source: reasoning.txt s10,
      technical-analysis-v3.txt Concern 3 Mechanism 1.
  - Add GrcError::InsufficientFindings { role: String, expected_min: usize }
  - Add GrcError::ConfigValidation { role: String, reason: String }
  - Define AgentGrcConfigLoader trait in reasoning/src/config.rs:
      fn load_config(role: &str, global_defaults: &GrcDefaults) -> Result<AgentGrcConfig, GrcError>
      fn validate_config(config: &AgentGrcConfig) -> Result<(), GrcError>
  - Define GrcDefaults: { budget: GrcBudget, grounding_level: GroundingLevel,
      csq_required: bool, allowed_methods: Vec<VerificationMethod> }
  - Stub: TomlConfigLoader reads from elai_cc.toml
  - Add per-role config routing to GrcEngine:
      fn with_agent_config(&mut self, config: AgentGrcConfig)
      fn get_agent_config(&self) -> Option<&AgentGrcConfig>
  - Re-export from reasoning/src/lib.rs
DETERMINISTIC_CHECK:
  - cargo check succeeds with 0 errors
    Command: cd .elai_cc && cargo check 2>&1 | grep -c "error\[" | grep -q "^0$"
  - AgentGrcConfig exists:
    Command: grep -c "pub struct AgentGrcConfig" .elai_cc/crates/reasoning/src/types.rs | grep -q "1"
  - InsufficientFindings error variant exists:
    Command: grep -q "InsufficientFindings" .elai_cc/crates/reasoning/src/types.rs
  - ConfigValidation error variant exists:
    Command: grep -q "ConfigValidation" .elai_cc/crates/reasoning/src/types.rs
  - Config loader trait exists:
    Command: grep -q "fn load_config\|fn validate_config" .elai_cc/crates/reasoning/src/config.rs
  - GrcEngine has with_agent_config method:
    Command: grep -q "fn with_agent_config" .elai_cc/crates/reasoning/src/engine.rs
ADVERSARIAL:
  - If AgentGrcConfig uses String for role instead of enum, typos undetected. Decision:
    String for now (cross-crate enum coupling worse), validate against known roles at
    load time.
  - If transition_overrides allows removing default transitions, misconfig could break
    base GRC chain. Test: validate overrides additive-only (union, not replacement).
ACCEPTANCE:
  - cargo check succeeds (workspace)
  - AgentGrcConfig has 8 fields (role, budget, required_steps, grounding_level,
    csq_required, allowed_methods, transition_overrides, require_scope_enumeration)
    Command: grep -A 10 "pub struct AgentGrcConfig" .elai_cc/crates/reasoning/src/types.rs | grep "pub " | wc -l
  - GrcError has InsufficientFindings and ConfigValidation variants

----------------------------------------------------------------
### 2C.2 VERIFY: compilation check
----------------------------------------------------------------
PRE_GATE:
  - Step 2C.1 complete (AgentGrcConfig type defined)
    Command: grep -q "pub struct AgentGrcConfig" .elai_cc/crates/reasoning/src/types.rs
TASK:
  - Run cargo check on full workspace.
  - Verify reasoning/ does not gain new crate deps (still core/ + 2B deps only).
  - Verify no new external dependencies added (toml parsing comes from core/ Config).
DETERMINISTIC_CHECK:
  - cargo check returns 0 errors
    Command: cd .elai_cc && cargo check 2>&1 | grep "error\[" | wc -l | grep -q "^0$"
  - No new dependency crates added:
    Command: diff <(git show HEAD:crates/reasoning/Cargo.toml) .elai_cc/crates/reasoning/Cargo.toml | grep "^+" | grep -v "^+++" | wc -l
ADVERSARIAL:
  - If reasoning/ depends on toml crate directly, bypasses core::Config. Test: reasoning/
    receives parsed AgentGrcConfig from orchestrator, not parses TOML itself.
ACCEPTANCE:
  - cargo check exit code 0
  - reasoning/ Cargo.toml has no toml/toml_edit dependency

----------------------------------------------------------------
### 2C.3 TEST: write failing tests (TDD)
----------------------------------------------------------------
PRE_GATE:
  - Step 2C.2 passes (workspace compiles clean)
  - AgentGrcConfig struct defined in types.rs
    Command: grep -q "pub struct AgentGrcConfig" .elai_cc/crates/reasoning/src/types.rs
TASK: Write 15 tests in reasoning/tests/agent_config.rs:
  T1. auditor_gets_double_verify: AgentGrcConfig auditor required_steps=
      [Observe,Hypothesize,Verify,Verify] and transition_overrides={Verify: [Verify,
      Observe]}. with_agent_config(); think(Observe), think(Hypothesize), think(Verify),
      think(Verify) all succeed; second Verify does NOT return InvalidTransition.
      Invariant: reasoning.txt s7 L97, s11 item 9.
  T2. context_compiler_observe_only: required_steps=[Observe]. think(Observe) succeeds.
      think(Hypothesize) => Err(InvalidTransition). Invariant: s7 L103-106.
  T3. executor_budget_8_1200: budget=(8, 1200). think() 8 times; 9th =>
      BudgetExhausted.
  T4. memory_writer_budget_2_400: budget=(2, 400). think() twice; 3rd =>
      BudgetExhausted.
  T5. config_validation_empty_steps_rejected: required_steps=[]. validate_config() =>
      Err(ConfigValidation) reason mentions empty steps.
  T6. config_validation_budget_too_small: budget.max_calls=2 with required_steps len 5.
      => Err(ConfigValidation) reason mentions budget insufficient for required chain.
  T7. auditor_anti_rubber_stamp: Run Observe, Hypothesize, Verify (no FINDING marker),
      Converge => Err(InsufficientFindings). Add Verify with "FINDING: unused variable",
      Converge => succeeds. Invariant: s7 L99.
  T8. auditor_rejects_uncertain_grounding: grounding_level=Verified.
      think(Observe) with grounding="training:unverified_claim" => error
      (Uncertain < Verified).
  T9. context_compiler_allowed_methods: allowed_methods=[FILE_CHECK, SOURCE_CHECK].
      Request WEB_CHECK => rejected.
  T10. global_defaults_with_per_role_override: GrcDefaults budget=(5,800). Executor
       overrides budget to (8,1200) inherits other defaults. Executor budget=(8,1200),
       other fields match defaults.
  T11. transition_overrides_are_additive: transition_overrides={Observe: [Verify]}.
       think(Observe). Next allowed should include BOTH Hypothesize (default) AND
       Verify (override). Not just Verify.
  T12. auditor_budget_10_1200: budget=(10,1200). think() 10 times; 11th =>
       BudgetExhausted. Invariant: s7 L96.
  T13. prompt_engineer_budget_5_800: budget=(5,800), required_steps=[Observe,
       Hypothesize, Converge]. think() 5 times; 6th => BudgetExhausted. Invariant:
       s7 L109-111.
  T14. designer_config_distinct_from_executor: Designer budget=(8,1200), required_steps=
       [Observe, Hypothesize, PlanAction, Verify, Converge], require_scope_enumeration=
       true. Designer differs from executor in role name, required_steps, role_prompt
       (Designer produces PipelineSpec, not code). Same model, separate AgentGrcConfig.
       Invariant: technical-analysis-v3.txt Concern 5, reasoning.txt s7.
  T15. require_scope_enumeration_per_role: executor true; memory_writer false.
       Executor's Observe requires scope; memory_writer's does not.
DETERMINISTIC_CHECK:
  - All 15 tests exist and FAIL (TDD)
    Command: cd .elai_cc && cargo test -p reasoning --test agent_config --no-run 2>&1
  - Test count:
    Command: grep -c "#\[test\]" .elai_cc/crates/reasoning/tests/agent_config.rs
    (should be >= 15)
ADVERSARIAL:
  - If tests pass against stubs, stubs too permissive. Use todo!() / unimplemented!().
  - If T7 uses exact string matching on "FINDING:", brittle. Use structured FindingMarker
    or case-insensitive prefix.
  - If T11 doesn't verify defaults preserved, additive-only merge could regress silently.
ACCEPTANCE:
  - >= 15 test functions exist
  - All tests fail to compile or fail at runtime (TDD contract)
  - Zero timing-dependent tests

----------------------------------------------------------------
### 2C.4 IMPLEMENT: config loading, per-role routing, transition extension
----------------------------------------------------------------
PRE_GATE:
  - Step 2C.3 complete (tests written, all currently fail)
  - No other session owns reasoning/ files
TASK:
  I1. Config validation (config.rs): validate_config() checks 5 rules from D2; returns
      Err(ConfigValidation) on first failure with specific reason.
  I2. Transition override merging + step whitelist (engine.rs):
      with_agent_config() stores config. On each think():
      effective_allowed_next(state) = base: GrcState::allowed_next(state)
      plus: config.transition_overrides.get(state).unwrap_or_default()
      merged: base.union(overrides) — additive only.
      THEN: if config.required_steps non-empty, filter merged to states in required_steps
      (plus Status and Converge, always allowed). This is the step whitelist.
      reachable_states = required_steps + [Status, Converge]
      effective_allowed = merged.intersect(reachable_states)
      Without filter, context-compiler's required_steps=[Observe] would have no effect
      (default allowed_next(Observe)=[Hypothesize] would still allow Hypothesize).
  I3. Grounding level enforcement (engine.rs): On think() with grounding, classify
      GroundingLevel. If level < config.grounding_level, reject. Implement PartialOrd.
      CRITICAL: enum variant order MUST be { Uncertain=0, High=1, Verified=2 } so derived
      PartialOrd gives Uncertain < High < Verified. Phase 2A's order
      { Verified, High, Uncertain } inverts: Verified=0 < Uncertain=2, check passes
      when should reject. Phase 2A's GroundingLevel order must be corrected or explicit
      PartialOrd impl provided (not derived). Test: assert!(GroundingLevel::Verified >
      GroundingLevel::Uncertain) — add to T8.
  I4. Anti-rubber-stamp (engine.rs): On think(Converge) when role == "auditor": scan
      session history for Verify steps with "FINDING:" prefix. count < 1 =>
      Err(InsufficientFindings { role, expected_min: 1 }). const FINDING_PREFIX:
      &str = "FINDING:". Contract: any Verify step whose content starts with
      FINDING_PREFIX (case-sensitive, after trimming leading whitespace). Internal
      protocol between GRC engine and auditor prompt — not natural language parsing.
      Tests must cover: exact prefix, leading whitespace, content without prefix.
  I5. Config loading from TOML (config.rs): TomlConfigLoader::load_config(role,
      defaults) reads from core::Config. Merges [grc.defaults] with [agents.{role}.grc]
      per D3. Calls validate_config() before returning. If role not found: defaults only
      (no error — allows dynamic roles). If found but validation fails: return error.
  I6. Per-role routing (engine.rs): with_agent_config() stores config and resets topic
      state. Subsequent think() uses config's budget, transitions, grounding level.
      get_agent_config() returns current config (orchestrator inspection). If no agent
      config: engine uses global defaults (backward-compatible with 2A).
      BUDGET ENFORCEMENT SCOPE: in phase 2C (in-process, pre-isolation), agent-side
      GrcEngine enforces budgets directly. In phase 4A (Tier C process isolation),
      orchestrator.txt L440-444: "Agent binary budget tracking is advisory only.
      Enforcement is in the orchestrator." Phase 2C's enforcement is AUTHORITATIVE for
      phases 0-3 (in-process). Phase 4A transitions authority to orchestrator and demotes
      agent-side to advisory. Same GrcBudget struct used in both modes; only who
      enforces changes.
DETERMINISTIC_CHECK:
  - All 15 tests pass
    Command: cd .elai_cc && cargo test -p reasoning --test agent_config -- --nocapture 2>&1 | grep -E "test result:" | grep -q "0 failed"
  - Plus all phase 2A and 2B tests still pass (no regression)
  - No todo!() or unimplemented!() in config.rs or engine.rs
    Command: grep -c "todo!\|unimplemented!" .elai_cc/crates/reasoning/src/config.rs .elai_cc/crates/reasoning/src/engine.rs | grep -v ":0$" | wc -l | grep -q "^0$"
  - No unwrap() in config.rs or engine.rs
  - GroundingLevel implements PartialOrd:
    Command: grep -q "PartialOrd" .elai_cc/crates/reasoning/src/types.rs
ADVERSARIAL:
  - If FINDING_PREFIX hardcoded string matching, model output format drift breaks
    detection. Decision: keep simple prefix (internal protocol). Document in code.
  - If with_agent_config() does not reset topic state, leftover state could leak. Test:
    set config A, run steps, set config B, verify clean.
  - If config loading fails silently (returns defaults on parse error), misconfig
    undetected. Test: malformed [agents.executor.grc] table must return error, not
    silently fall back. Only missing table falls back.
  - If PartialOrd on GroundingLevel wrong (Uncertain > Verified), auditor accepts
    untested claims. Test: assert Verified > High > Uncertain explicitly.
ACCEPTANCE:
  - cargo test -p reasoning: 0 failures, all 2A + 2B + 2C tests pass
  - cargo check --workspace: 0 errors
  - grep todo!/unimplemented!/unwrap() in engine.rs and config.rs: 0 matches
  - GroundingLevel has PartialOrd
  - FINDING_PREFIX constant defined

----------------------------------------------------------------
### 2C.5 MEASURE: config overhead and correctness metrics
----------------------------------------------------------------
PRE_GATE: Step 2C.4 complete (all tests pass)
TASK:
  - Config load + validation time per role target: <1ms
  - Per-step overhead from config routing vs no-config baseline target: <10%
  - Memory per AgentGrcConfig target: <2KB including transition_overrides
  - Record: test count, LOC delta from phase 2B, compile time delta
  - Write metrics to .elai_cc/metrics/phase_2C.toml
DETERMINISTIC_CHECK:
  - Metrics file exists and contains required fields
    Command: test -f .elai_cc/metrics/phase_2C.toml && grep -q "config_load_us" .elai_cc/metrics/phase_2C.toml
  - Config load overhead < 1000us
    Command: grep "config_load_us" .elai_cc/metrics/phase_2C.toml (value < 1000)
  - Per-step routing overhead < 10%
    Command: grep "routing_overhead_pct" .elai_cc/metrics/phase_2C.toml (value < 10.0)
ADVERSARIAL:
  - If measurement includes TOML parsing time, conflates I/O. Test: measure config
    validation separately. Report both.
  - If AgentGrcConfig with large transition_overrides uses excessive memory, <2KB target
    may fail. Test: measure with max reasonable overrides (7x7=49 entries).
ACCEPTANCE:
  - .elai_cc/metrics/phase_2C.toml exists with all required fields
  - config_load_us < 1000
  - routing_overhead_pct < 10.0
  - cargo test -p reasoning: 0 failures

EXIT CRITERIA (Phase 2C)
  Each of the 6 agent roles (including Designer) has correct GRC budget and step sequence.
  Designer role has distinct AgentGrcConfig from Executor (same model, different
  role_prompt/steps).
  Auditor gets double-verify via transition_overrides (verify->verify allowed).
  Context-compiler gets observe-only (hypothesize/plan_action/act/verify rejected).
  Auditor anti-rubber-stamp enforced: converge without finding is rejected.
  Grounding level enforcement: auditor rejects Uncertain, context-compiler accepts High.
  Config validation catches: empty steps, budget < steps, missing verify for csq,
  invalid methods.
  Global defaults + per-role overrides merge correctly (field-by-field, not deep).
  Transition overrides are additive-only (default transitions preserved).
  require_scope_enumeration enforced per-role (true for executor/designer, false for
  memory_writer).
  All phase 2A + 2B tests still pass (no regression).
  All 15+ phase 2C tests pass. Zero todo!/unimplemented!/unwrap() in new code.

GROUNDING SUMMARY (Phase 2C)
  "Auditor required_steps includes double-verify" -> GROUNDED: reasoning.txt s7 L97
  "Context-compiler is observe-only with FILE_CHECK/SOURCE_CHECK" -> GROUNDED: s7 L103-106
  "Executor budget 8/1200, memory-writer 2/400" -> GROUNDED: s7 L89-91, L115-116
  "Global default configurable in elai_cc.toml" -> GROUNDED: s3 L38
  "[agents.{role}] TOML pattern" -> VERIFIED: 5 specs use this pattern
  "Anti-rubber-stamp: must find >= 1 issue" -> GROUNDED: s7 L99
  "Per-agent FSM sub-states: IDLE->LOADING->REASONING->ACTING->DONE" -> GROUNDED: s7 L118-120
  "AgentGrcConfig type has 4 fields" -> GROUNDED: s10 L186. GAP: missing
    allowed_methods, transition_overrides, require_scope_enumeration. Now 8 fields total.
  "require_scope_enumeration field on AgentGrcConfig" -> GROUNDED: s10,
    technical-analysis-v3.txt Concern 3 Mechanism 1
  "Designer is a distinct AgentRole from Executor" -> GROUNDED:
    technical-analysis-v3.txt Concern 5, reasoning.txt s7
  "Orchestrator enforces budget externally via IPC message counting" -> GROUNDED:
    orchestrator.txt L440-444
  "Per-agent config routing prevents InvalidTransition" -> VERIFIED
  "Config validation at load time (fail-fast)" -> GROUNDED
  "Anti-rubber-stamp defeats adversarial review if advisory-only" -> GROUNDED
  "Role-based agent config is standard pattern" -> VERIFIED: MetaGPT, CrewAI 2025
  "<1ms config load overhead" -> UNGROUNDED
  "<10% per-step routing overhead" -> UNGROUNDED

SPEC GAPS IDENTIFIED (for reconciliation with other sessions)
  1. reasoning.txt L186: AgentGrcConfig should add allowed_methods: Vec<VerificationMethod>,
     transition_overrides: HashMap<GrcState, Vec<GrcState>>, require_scope_enumeration: bool.
     Currently 4 fields, needs 8. (reasoning.txt owned by session s-1003192.)
  2. reasoning.txt s11: testable invariant for anti-rubber-stamp not listed. Should add:
     "[2C] Auditor converge without finding is rejected (InsufficientFindings)."
  3. reasoning.txt GrcError enum (L174): missing InsufficientFindings and ConfigValidation
     variants.
  4. reasoning.txt s10 (GroundingLevel enum): variant order must be { Uncertain, High,
     Verified } (ascending quality = ascending discriminant) for derived PartialOrd.
     Current order { Verified, High, Uncertain } inverts comparison. Phase 2A must fix
     before phase 2C can use PartialOrd for grounding level enforcement.
  5. reasoning.txt s7: grounding_level field semantic ambiguity. allowed_methods restricts
     WHICH methods, grounding_level restricts MINIMUM QUALITY.

================================================================
## PHASE 2D: CSQ AND INLINE FEASIBILITY
================================================================
Prerequisites: Phase 2C complete (per-agent GRC configs route correctly, all 2C tests pass)
SCOPE: see docs/specs/guide/implementation-guide.txt

GOAL
Counterfactual Safety Questions (CSQ) on functions touching files/data/network/auth.
Inline feasibility checks catch ungrounded claims during EXECUTING (max 2 GRC calls).
After this phase, the reasoning engine is complete (phases 2A-2D).

READ BEFORE IMPLEMENTING
  docs/specs/crates/reasoning.txt             Section 6 (CSQ: format, triggers, guard requirement)
  docs/specs/crates/reasoning.txt             Section 8 (inline feasibility: detection, max 2 calls, BLOCK signal)
  docs/specs/crates/reasoning.txt             Section 7 (per-agent CSQ config: which roles require CSQ)
  docs/specs/crates/reasoning.txt             Section 10 (KEY TYPES: items marked [2D])
  docs/specs/crates/reasoning.txt             Section 11 (TESTABLE INVARIANTS: items marked [2D])
  docs/specs/crates/orchestrator.txt          Section 8 (FSM transitions: EXECUTING->PLANNING replan signals)
  docs/specs/architecture-overview.txt        Section 4 (FSM flow: EXECUTING includes GRC inline feasibility)

DESIGN DECISIONS (grounded)
  D1. CSQ trigger detection via function signature analysis. CSQ fires on functions
      touching files/network/money/auth. Detection: pattern match on parameters and
      return types. Functions receiving Path/File/TcpStream/Money/AuthToken or calling
      std::fs/std::net APIs => CSQ-required. Phase 1A DKG provides call-graph; CSQ
      detection works WITHOUT DKG (reasoning crate depends on core+providers+tools, not
      knowledge). Standalone pattern: caller passes "sensitive function signatures" list
      from DKG at runtime. Grounding: reasoning.txt s6, ast-grep
      (github.com/ast-grep/ast-grep). Reasoning/ depends on core/, providers/, tools/
      (NOT knowledge/). DKG queries injected by orchestrator at call time.
  D2. CSQ severity classification determines guard enforcement.
      CRITICAL (guard MANDATORY): data_loss, security_breach, financial_error, auth_bypass
      WARNING (guard RECOMMENDED): incorrect_output, performance_degradation, stale_cache
      CRITICAL with no guard added = CSQ failure (blocks AUDITING).
      WARNING with no guard = CSQ warning (logged, does not block).
      Grounding: reasoning.txt s6 "Dangerous consequence (data loss, security, financial)
      -> MUST add guard." VeriGuard (OpenReview 2025) co-generates verification with code.
  D3. Inline feasibility uses GRC budget counter, not separate mechanism. Inline
      feasibility (s8) fires when GRC verify detects NEW claim not in PipelineSpec /
      Research Report / DKG. Max-2-call limit enforced by dedicated counter in
      InlineFeasibilityState, separate from per-topic GRC budget. Rationale: inline
      checks are cross-topic. Methods: web_search, grep, api_registry_check (s8).
      Grounding: reasoning.txt s8 "Quick check (max 2 GRC calls)." AgentSpec
      (arxiv:2503.18666). Receiver note: check_inline_feasibility takes &mut self
      (mutates counter). Prior GrcEngine methods (think, set_budget) already use
      &mut self. verify_claim/ground_plan (2B) use &self since read-only.
  D4. Inline feasibility failure signals replan via ReplanRequest.
      PASS: claim grounded, proceed.
      FAIL: executor uses plan's original approach (no deviation).
      BLOCKED (fundamentally different approach): executor emits ReplanRequest(MAJOR)
      => EXECUTING->PLANNING transition requiring human approval (orchestrator.txt s8:
      "EXECUTING -> PLANNING (major replan): requires human approval in both AUTONOMOUS
      and INTERACTIVE modes"). Grounding: reasoning.txt s8, orchestrator.txt s8.
  D5. CSQ and inline feasibility types extend reasoning/src/types.rs.
      CsqCheck { assumption: String, consequence: String, severity: CsqSeverity,
                  guard_added: bool }
      CsqSeverity enum { Critical, Warning }
      CsqResult { function_id: String, checks: Vec<CsqCheck> }
        fn passed(&self) -> bool — true iff all CRITICAL checks have guard_added==true.
        No stored passed field; computed method ensures consistency.
      FeasibilityMethod enum { SourceMatch }
        Typed enum following established VerificationMethod pattern (reasoning.txt s10).
        Extensible: future variants WebSearch, Grep, ApiRegistryCheck.
      InlineFeasibilityResult { claim: String, method: FeasibilityMethod,
                                 outcome: FeasibilityOutcome }
      FeasibilityOutcome enum { Pass, Fail, Blocked }
        Pass: verified, Fail: use original plan, Blocked: emit ReplanRequest(MAJOR).
      InlineFeasibilityState { checks_used: u32, max_checks: u32 }
        max_checks = 2 (hardcoded from spec). is_exhausted() = checks_used >= max_checks.
        reset() sets checks_used = 0 (orchestrator calls between phases).
      Grounding: type fields derived from reasoning.txt s6/s8.

----------------------------------------------------------------
### 2D.1 SKELETON: CSQ and inline feasibility types + stubs
----------------------------------------------------------------
PRE_GATE:
  - Phase 2C tests pass (per-agent GRC configs work)
    Command: cd .elai_cc && cargo test -p reasoning -- --nocapture 2>&1 | grep -E "test result:" | grep -q "0 failed"
  - AgentGrcConfig type exists with csq_required field (from phase 2C)
    Command: grep -q "csq_required" .elai_cc/crates/reasoning/src/types.rs && echo PASS
  - No CsqCheck type exists yet (greenfield for 2D types)
    Command: grep -c "CsqCheck" .elai_cc/crates/reasoning/src/types.rs | grep -q "^0$" && echo PASS
TASK:
  - Add types to reasoning/src/types.rs:
      CsqSeverity enum { Critical, Warning }
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
      CsqCheck { assumption: String, consequence: String,
                 severity: CsqSeverity, guard_added: bool }
        #[derive(Debug, Clone)]
      CsqResult { function_id: String, checks: Vec<CsqCheck> }
        #[derive(Debug, Clone)]
        fn passed(&self) -> bool { self.checks.iter().all(|c| c.severity != Critical || c.guard_added) }
        No stored passed field — computed method ensures consistency.
      InlineFeasibilityState { checks_used: u32, max_checks: u32 }
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        Default: max_checks = 2
        fn is_exhausted(&self) -> bool { self.checks_used >= self.max_checks }
        fn record_check(&mut self) { self.checks_used = self.checks_used.saturating_add(1); }
        fn reset(&mut self) { self.checks_used = 0; }  // orchestrator calls between phases
      FeasibilityOutcome enum { Pass, Fail, Blocked }
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
      FeasibilityMethod enum { SourceMatch }
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        Follows VerificationMethod pattern (reasoning.txt s10 L190, L194).
        Future variants: WebSearch, Grep, ApiRegistryCheck (added in phase 4F).
      InlineFeasibilityResult { claim: String, method: FeasibilityMethod,
                                outcome: FeasibilityOutcome }
        #[derive(Debug, Clone)]
  - Add methods to GrcEngine trait in reasoning/src/engine.rs:
      Phase 2D methods (implemented this phase):
        fn generate_csq(&self, function_id: &str, sensitive_ops: &[String])
          -> Result<CsqResult, GrcError>
        fn check_inline_feasibility(&mut self, claim: &str, sources: &[String])
          -> Result<InlineFeasibilityResult, GrcError>
      Stubs in BasicGrcEngine using todo!() until step 2D.4.
  - Add InlineFeasibilityState field to BasicGrcEngine struct.
  - Re-export new types from reasoning/src/lib.rs
DETERMINISTIC_CHECK:
  - cargo check on workspace succeeds with 0 errors
    Command: cd .elai_cc && cargo check 2>&1 | grep -c "error\[" | grep -q "^0$"
  - All new types exist:
    Command: grep -c "pub enum CsqSeverity" .elai_cc/crates/reasoning/src/types.rs | grep -q "1"
    Command: grep -c "pub struct CsqCheck" .elai_cc/crates/reasoning/src/types.rs | grep -q "1"
    Command: grep -c "pub struct CsqResult" .elai_cc/crates/reasoning/src/types.rs | grep -q "1"
    Command: grep -c "pub struct InlineFeasibilityState" .elai_cc/crates/reasoning/src/types.rs | grep -q "1"
    Command: grep -c "pub enum FeasibilityOutcome" .elai_cc/crates/reasoning/src/types.rs | grep -q "1"
    Command: grep -c "pub enum FeasibilityMethod" .elai_cc/crates/reasoning/src/types.rs | grep -q "1"
    Command: grep -c "pub struct InlineFeasibilityResult" .elai_cc/crates/reasoning/src/types.rs | grep -q "1"
  - GrcEngine trait has new methods:
    Command: grep -c "fn generate_csq" .elai_cc/crates/reasoning/src/engine.rs | grep -q "1"
    Command: grep -c "fn check_inline_feasibility" .elai_cc/crates/reasoning/src/engine.rs | grep -q "1"
ADVERSARIAL:
  - If CsqCheck.guard_added defaults to true, all CSQ auto-pass without actual guards.
    Test: CsqCheck must require explicit guard_added=true from caller. No Default impl
    sets guard_added=true.
  - If InlineFeasibilityState.max_checks configurable, callers could set to u32::MAX.
    Test: max_checks hardcoded to 2 in Default impl, not settable via public constructor.
ACCEPTANCE:
  - cargo check succeeds: exit code 0
  - 7 new types exist (CsqSeverity, CsqCheck, CsqResult, InlineFeasibilityState,
    FeasibilityMethod, FeasibilityOutcome, InlineFeasibilityResult)
  - 2 new methods in GrcEngine trait (generate_csq, check_inline_feasibility)

----------------------------------------------------------------
### 2D.2 VERIFY: compilation check
----------------------------------------------------------------
PRE_GATE:
  - Step 2D.1 complete (new types and stubs exist)
    Command: grep -q "CsqCheck" .elai_cc/crates/reasoning/src/types.rs && echo PASS
TASK:
  - Run cargo check on full workspace.
  - Verify reasoning/ has no NEW deps beyond declared (core, providers, tools, thiserror,
    serde).
DETERMINISTIC_CHECK:
  - cargo check returns 0 errors
    Command: cd .elai_cc && cargo check 2>&1 | grep "error\[" | wc -l | grep -q "^0$"
  - No new dependencies beyond declared set:
    Command: grep "^\[dependencies" .elai_cc/crates/reasoning/Cargo.toml -A 30 | grep -v "core" | grep -v "providers" | grep -v "tools" | grep -v "^\[" | grep -v "^$" | grep -v "thiserror" | grep -v "serde" | grep -v "#" | wc -l
ADVERSARIAL:
  - If stubs return Ok(default_value) instead of todo!(), tests might pass falsely in
    2D.3 TDD. Test: grep todo!() in generate_csq and check_inline_feasibility.
ACCEPTANCE:
  - cargo check exit code 0
  - reasoning/ dependency set: only core, providers, tools, thiserror, serde

----------------------------------------------------------------
### 2D.3 TEST: write failing tests (TDD)
----------------------------------------------------------------
PRE_GATE:
  - Step 2D.2 passes (workspace compiles clean)
  - generate_csq and check_inline_feasibility methods exist in GrcEngine trait
    Command: grep -q "fn generate_csq" .elai_cc/crates/reasoning/src/engine.rs
    Command: grep -q "fn check_inline_feasibility" .elai_cc/crates/reasoning/src/engine.rs
TASK: Write 10 tests in reasoning/tests/csq_tests.rs:
  T1. csq_generates_check_for_file_function: generate_csq("write_config",
      &["std::fs::write"]) => result.checks.len() >= 1; at least one check has
      severity == CsqSeverity::Critical. Invariant: reasoning.txt s6.
  T2. csq_critical_without_guard_fails: CsqResult with one Critical check guard_added==
      false; result.passed() == false. Invariant: s6.
  T3. csq_critical_with_guard_passes: CsqResult Critical guard_added==true;
      result.passed() == true.
  T4. csq_warning_without_guard_still_passes: Warning, guard_added==false;
      result.passed == true (warnings don't block).
  T5. csq_no_sensitive_ops_returns_empty: generate_csq("helper_fn", &[]) =>
      checks.is_empty(); passed() == true.
  T6. inline_feasibility_pass_on_known_claim: check_inline_feasibility(
      "use tantivy BM25 index for full-text search", &["PipelineSpec: use tantivy BM25
      index for full-text search"]) => outcome == Pass. Note: realistic multi-word claim
      avoids trivial substring false positive.
  T7. inline_feasibility_fail_on_unknown_claim: check_inline_feasibility("unknown claim",
      &[]) => outcome == Fail.
  T8. inline_feasibility_max_two_calls: assert engine.inline_feasibility_state.checks_used
      == 0 (initial). check_inline_feasibility() twice. assert checks_used == 2. Third
      call => Err(GrcError::BudgetExhausted). Invariants: s8 "max 2 GRC calls"; s11 "[2D]
      Inline feasibility max 2 calls (bounded)."
  T9. inline_feasibility_blocked_signals_replan: check_inline_feasibility(
      "CONTRADICTS_PLAN: use completely different algorithm", &["use original algorithm"])
      => outcome == Blocked. CONTRADICTS_PLAN: prefix is phase 2D trigger for Blocked
      (I2). Invariant: s8 "Fundamentally different approach requires EXECUTING -> BLOCK
      -> PLANNING." Note: Blocked outcome signals need for ReplanRequest(MAJOR); actual
      ReplanRequest emission is orchestrator's responsibility (3A+).
  T10. inline_feasibility_state_reset: After exhausting (2 used), reset on
       InlineFeasibilityState. checks_used == 0; is_exhausted() == false. Ensures clean
       between phases/topics.
DETERMINISTIC_CHECK:
  - All 10 tests exist and FAIL (TDD)
    Command: cd .elai_cc && cargo test -p reasoning --test csq_tests --no-run 2>&1 | grep -q "error"
  - Test count matches:
    Command: grep -c "#\[test\]" .elai_cc/crates/reasoning/tests/csq_tests.rs
    (should be >= 10)
ADVERSARIAL:
  - If tests use hardcoded strings matching stub defaults, false pass. Stubs must use
    todo!() / unimplemented!().
  - If T8 uses shared mutable counter without proper scoping, test isolation breaks.
    Test: each test creates its own BasicGrcEngine instance.
ACCEPTANCE:
  - >= 10 test functions exist
  - All tests fail to compile or fail at runtime (TDD contract)
  - Zero timing-dependent tests

----------------------------------------------------------------
### 2D.4 IMPLEMENT: CSQ generation and inline feasibility logic
----------------------------------------------------------------
PRE_GATE:
  - Step 2D.3 complete (tests written, all currently fail)
  - No other session owns reasoning/ files
TASK: Implement in BasicGrcEngine (reasoning/src/engine.rs):
  I1. CSQ generation (generate_csq):
      Input: function_id, sensitive_ops list.
      Logic:
        - If sensitive_ops empty, return CsqResult { function_id, checks: vec![] }.
          (passed() returns true since no Critical checks exist.)
        - For each sensitive_op, generate a CsqCheck:
          - Classify severity: ops matching file/network/money/auth -> Critical, others
            -> Warning.
          - Pattern matching (case-insensitive: lowercase both sensitive_op and pattern):
            CRITICAL patterns: "fs::", "file", "write", "delete", "remove", "net::",
              "tcpstream", "money", "payment", "auth", "token", "credential", "password"
            WARNING patterns: everything else.
          - assumption: "The call to {sensitive_op} in {function_id} behaves as expected"
          - consequence (template):
            Critical: "data loss/security breach/financial error possible if
              {sensitive_op} fails"
            Warning: "incorrect output possible if {sensitive_op} fails"
          - guard_added: false (caller sets to true after adding guard).
        - Return CsqResult { function_id, checks }. passed() computes dynamically:
          checks.iter().all(|c| c.severity != Critical || c.guard_added)
  I2. Inline feasibility check (check_inline_feasibility):
      Input: claim, sources list.
      Logic:
        - Check InlineFeasibilityState.is_exhausted(). If yes => Err(BudgetExhausted)
          with topic="inline_feasibility".
        - Check claim in any source (substring or exact). Found: outcome = Pass.
        - Not found: outcome = Fail (use original approach).
        - Blocked detection: claim contains negation of plan assumption (heuristic:
          claim and source share >50% tokens but contradicting verb/predicate).
          Note: heuristic intentionally simple for 2D. Phase 4F (FEASIBILITY_PROBE)
          adds full semantic analysis. For now, Blocked only when explicitly signaled
          by caller via claim prefixed "CONTRADICTS_PLAN:".
        - Record check: self.inline_feasibility_state.record_check().
        - Return InlineFeasibilityResult { claim, method: FeasibilityMethod::SourceMatch,
          outcome }.
  I3. CsqResult.passed() method:
      fn passed(&self) -> bool { self.checks.iter().all(|c| c.severity != Critical ||
        c.guard_added) }
      Always consistent with current check state. No stored passed field.
DETERMINISTIC_CHECK:
  - All 10 tests pass
    Command: cd .elai_cc && cargo test -p reasoning --test csq_tests -- --nocapture 2>&1 | grep -E "test result:" | grep -q "0 failed"
  - No todo!() or unimplemented!() in CSQ/inline methods
    Command: grep -A 20 "fn generate_csq" .elai_cc/crates/reasoning/src/engine.rs | grep -c "todo!\|unimplemented!" | grep -q "^0$"
    Command: grep -A 20 "fn check_inline_feasibility" .elai_cc/crates/reasoning/src/engine.rs | grep -c "todo!\|unimplemented!" | grep -q "^0$"
  - No unwrap() in CSQ/inline implementation
    Command: grep -A 20 "fn generate_csq" .elai_cc/crates/reasoning/src/engine.rs | grep -c "\.unwrap()" | grep -q "^0$"
  - All prior phase 2A-2C tests still pass
ADVERSARIAL:
  - If CRITICAL pattern list case-sensitive, "FS::" or "Net::" bypasses CSQ. Test: mixed-
    case sensitive_ops ("std::FS::write") => Critical. Compare lowercase.
  - If inline feasibility substring match too loose, "the" in claim matches every source
    => false Pass. Test: meaningful claim content (>3 words or exact match), not single
    common words.
  - If Blocked detection false-positives on similar but non-contradicting claims,
    executor stuck in replan loops. Test: "uses std::fs::write" vs "uses std::fs::read"
    should NOT trigger Blocked.
  - If InlineFeasibilityState shared across topics, exhausting in one phase blocks all
    subsequent. Test: state resets between phases (reset method called by orchestrator).
ACCEPTANCE:
  - cargo test -p reasoning: 0 failures, all 2A-2D tests pass
  - cargo check --workspace: 0 errors
  - grep todo!/unimplemented!/unwrap() in CSQ+inline methods: 0 matches
  - At least 10 tests pass in csq_tests.rs

----------------------------------------------------------------
### 2D.5 MEASURE: CSQ and inline feasibility metrics
----------------------------------------------------------------
PRE_GATE: Step 2D.4 complete (all tests pass)
TASK:
  - CSQ generation overhead per function target: <1ms for pattern matching.
  - Inline feasibility check overhead target: <2ms for source matching.
  - False positive rate on inline checks: 20 claims (10 present, 10 absent). Assert
    0 false positives, 0 false negatives. Pattern matching deterministic: 0%.
  - Record: test count delta from phase 2C, LOC delta, compile time delta.
  - Write metrics to .elai_cc/metrics/phase_2D.toml.
  - Update architecture-overview.txt section 4 GrcEngine trait listing to include
    generate_csq and check_inline_feasibility (prevents spec drift).
DETERMINISTIC_CHECK:
  - Metrics file exists with required fields
    Command: test -f .elai_cc/metrics/phase_2D.toml && grep -q "csq_overhead_us" .elai_cc/metrics/phase_2D.toml
    Command: grep -q "inline_check_overhead_us" .elai_cc/metrics/phase_2D.toml
    Command: grep -q "false_positive_rate" .elai_cc/metrics/phase_2D.toml
  - CSQ overhead < 1000us
    Command: grep "csq_overhead_us" .elai_cc/metrics/phase_2D.toml (value < 1000)
  - False positive rate = 0
    Command: grep "false_positive_rate" .elai_cc/metrics/phase_2D.toml | grep -q "0.0"
ADVERSARIAL:
  - If measurement uses wall-clock in CI, non-deterministic. Use std::hint::black_box +
    std::time::Instant local benchmarks only. Median 1000 iterations. NOT criterion
    without user approval.
  - If false_positive_rate test uses trivial claims ("a","b"), not real-world. Use
    realistic claim strings (>5 words each).
ACCEPTANCE:
  - .elai_cc/metrics/phase_2D.toml exists with all required fields
  - csq_overhead_us < 1000
  - inline_check_overhead_us < 2000
  - false_positive_rate == 0.0
  - cargo test -p reasoning: 0 failures

EXIT CRITERIA (Phase 2D)
  CSQ fires on every function with sensitive operations (file/network/money/auth).
  CSQ with Critical severity and no guard = failure (blocks progression).
  CSQ with Warning severity and no guard = warning only (does not block).
  Inline feasibility checks bounded at max 2 per EXECUTING phase.
  Inline feasibility Pass = proceed, Fail = use original plan, Blocked = replan.
  All 2D tests pass. All prior 2A-2C tests still pass (regression).
  Zero todo!/unimplemented!/unwrap() in CSQ and inline feasibility code.

PHASE 2 OVERALL: reasoning engine complete (2A state machine + 2B grounding verification
+ 2C per-agent config + 2D CSQ + inline feasibility).

GROUNDING SUMMARY (Phase 2D)
  "CSQ: if [X] wrong, code will [Y]" -> GROUNDED: reasoning.txt s6
  "Min 1 CSQ per file/network/money/auth function" -> GROUNDED: s6
  "Dangerous consequence = MUST add guard" -> GROUNDED: s6
  "Inline feasibility max 2 GRC calls" -> GROUNDED: s8
  "NEW claim not in PipelineSpec/Research Report/DKG triggers check" -> GROUNDED: s8
  "PASS->proceed, FAIL->original approach" -> GROUNDED: s8
  "Fundamentally different approach -> EXECUTING->BLOCK->PLANNING" -> GROUNDED: s8,
    orchestrator.txt s8 (MAJOR replan path)
  "Per-agent CSQ config: executor=mandatory, auditor=mandatory, compiler=none" ->
    GROUNDED: s7
  "CSQ severity classification (CRITICAL vs WARNING)" -> VERIFIED: derived from s6
    "dangerous consequence"; VeriGuard (OpenReview 2025)
  "Detection via function signature pattern matching" -> VERIFIED: ast-grep
  "Runtime enforcement with bounded checks" -> VERIFIED: AgentSpec (arxiv:2503.18666)
  "InlineFeasibilityState separate from per-topic GRC budget" -> GROUNDED: s8 cross-topic
  "FeasibilityMethod enum instead of String" -> GROUNDED: s10 L190, L194
  "<1ms CSQ overhead, <2ms inline check overhead" -> UNGROUNDED
  "Blocked detection via CONTRADICTS_PLAN: prefix" -> UNGROUNDED: simplified heuristic;
    full semantic contradiction detection deferred to phase 4F (FEASIBILITY_PROBE)

================================================================
## CROSS-PHASE COLLECTED METADATA
================================================================

FILES TOUCHED (cumulative across Phase 2)
  .elai_cc/Cargo.toml (workspace member registration)
  .elai_cc/crates/reasoning/Cargo.toml
  .elai_cc/crates/reasoning/src/lib.rs
  .elai_cc/crates/reasoning/src/types.rs
  .elai_cc/crates/reasoning/src/engine.rs
  .elai_cc/crates/reasoning/src/verification.rs       (2B onwards)
  .elai_cc/crates/reasoning/src/config.rs             (2C onwards)
  .elai_cc/crates/reasoning/tests/*.rs                (12 phase 2A tests)
  .elai_cc/crates/reasoning/tests/verification_tests.rs   (16 phase 2B tests)
  .elai_cc/crates/reasoning/tests/agent_config.rs         (15 phase 2C tests)
  .elai_cc/crates/reasoning/tests/csq_tests.rs            (10 phase 2D tests)
  .elai_cc/metrics/phase_2A.toml
  .elai_cc/metrics/phase_2B.toml
  .elai_cc/metrics/phase_2C.toml
  .elai_cc/metrics/phase_2D.toml
  docs/specs/architecture-overview.txt   (update in 2D.5: GrcEngine trait listing)

TYPES / STRUCTS / TRAITS / ENUMS INTRODUCED
  Phase 2A:
    enum GrcState { Observe, Hypothesize, PlanAction, Act, Verify, Converge, Status }
      fn allowed_next(&self) -> &[GrcState]
    struct ThinkStep { step_type: GrcState, content: String, topic: String,
      grounding: Option<String>, conclusion: Option<String>, scope: Option<Vec<String>> }
    struct ThinkResult { step: ThinkStep, budget_remaining: GrcBudget,
      grounding_level: Option<GroundingLevel>, warnings: Vec<String> }
    struct GrcBudget { max_calls: u32, max_chars: u32, calls_used: u32, chars_used: u32 }
      fn remaining_calls(&self) -> u32
      fn remaining_chars(&self) -> u32
      fn is_exhausted(&self) -> bool
      fn must_converge(&self) -> bool
    enum GroundingLevel { Verified, High, Uncertain }   // NOTE: 2C requires reorder to
      { Uncertain, High, Verified } for derived PartialOrd
    enum GrcError { UngroundedStep{step_type,topic},
                    BudgetExhausted{topic,calls_used,max_calls},
                    InvalidTransition{from: GrcState, to: GrcState, allowed: Vec<GrcState>},
                    MalformedGrounding{raw,reason} }
    trait GrcEngine (6 methods total: 4 implemented in 2A + 2 stubs for 2B)
      fn think(&mut self, step: ThinkStep) -> Result<ThinkResult, GrcError>
      fn set_budget(&mut self, topic: &str, max_calls: u32, max_chars: u32)
      fn get_budget(&self, topic: &str) -> Option<&GrcBudget>
      fn reset_topic(&mut self, topic: &str)
      fn verify_claim(&self, claim: &str, source: &str) -> Result<VerificationResult, GrcError>   // stub
      fn ground_plan(&self, plan: &str) -> Vec<GroundingResult>                                   // stub
    struct BasicGrcEngine { topics: HashMap<String, TopicState>, ... }
    struct TopicState { budget: GrcBudget, last_state: Option<GrcState>,
      steps: Vec<ThinkStep>, grounding_log: Vec<(String, GroundingLevel)> }
    struct VerificationResult (stub) { claim, passes, evidence, method_used: String }
    struct GroundingResult (stub) { claim, method: String, level, source }
  Phase 2B:
    enum VerificationMethod { FileCheck, SourceCheck, TestCheck, WebCheck, ApiCheck,
      ApiRegistryCheck }
      fn requires_network(&self) -> bool
      fn requires_tools(&self) -> bool
    struct PlanGroundingResult { claims: Vec<GroundingResult>, all_grounded: bool,
      ungrounded: Vec<String>, verify_needed: Vec<String> }
    struct VerificationResult (updated) { claim, passes, evidence,
      method_used: VerificationMethod }
    struct GroundingResult (updated) { claim, method: VerificationMethod, level, source,
      verified_at: u64 }
    enum GrcError extension: VerificationError { method: VerificationMethod,
      claim: String, reason: String }
    trait Verifier: Send + Sync {
      fn verify(&self, method: VerificationMethod, claim: &str, context: &str)
        -> Result<VerificationResult, GrcError>
      fn supported_methods(&self) -> &[VerificationMethod]
    }
    trait TestRunner: Send + Sync {
      fn run_test(&self, name: &str, crate_name: &str) -> TestRunResult
    }
    struct TestRunResult { passed: bool, output: String, duration_ms: u64 }
    struct CargoTestRunner { sandbox: Box<dyn BashSandbox>, timeout_ms: u64 }
    struct MockTestRunner { results: HashMap<String, TestRunResult> }
    struct LocalVerifier (impl Verifier; takes Box<dyn TestRunner>)
    struct NetworkVerifier (impl Verifier; stub returns VerificationError)
    GrcEngine trait signature changes:
      fn verify_claim(&self, claim: &str, source: &str, context: &str)
        -> Result<VerificationResult, GrcError>   // adds context arg
      fn ground_plan(&self, plan: &str) -> PlanGroundingResult   // changes return type
  Phase 2C:
    struct AgentGrcConfig {
      role: String,
      budget: GrcBudget,
      required_steps: Vec<GrcState>,
      grounding_level: GroundingLevel,
      csq_required: bool,
      allowed_methods: Vec<VerificationMethod>,
      transition_overrides: HashMap<GrcState, Vec<GrcState>>,
      require_scope_enumeration: bool,
    }
    struct GrcDefaults { budget: GrcBudget, grounding_level: GroundingLevel,
      csq_required: bool, allowed_methods: Vec<VerificationMethod> }
    enum GrcError extension:
      InsufficientFindings { role: String, expected_min: usize }
      ConfigValidation { role: String, reason: String }
    trait AgentGrcConfigLoader {
      fn load_config(role: &str, global_defaults: &GrcDefaults)
        -> Result<AgentGrcConfig, GrcError>
      fn validate_config(config: &AgentGrcConfig) -> Result<(), GrcError>
    }
    struct TomlConfigLoader (impl AgentGrcConfigLoader)
    GrcEngine trait additions:
      fn with_agent_config(&mut self, config: AgentGrcConfig)
      fn get_agent_config(&self) -> Option<&AgentGrcConfig>
    const FINDING_PREFIX: &str = "FINDING:"
    GroundingLevel: must derive (or impl) PartialOrd, Ord with order Uncertain < High <
      Verified.
  Phase 2D:
    enum CsqSeverity { Critical, Warning }
    struct CsqCheck { assumption: String, consequence: String,
      severity: CsqSeverity, guard_added: bool }
    struct CsqResult { function_id: String, checks: Vec<CsqCheck> }
      fn passed(&self) -> bool   // computed; checks.iter().all(|c|
                                 //   c.severity != Critical || c.guard_added)
    struct InlineFeasibilityState { checks_used: u32, max_checks: u32 }
      fn is_exhausted(&self) -> bool
      fn record_check(&mut self)
      fn reset(&mut self)
    enum FeasibilityOutcome { Pass, Fail, Blocked }
    enum FeasibilityMethod { SourceMatch }   // future: WebSearch, Grep, ApiRegistryCheck
    struct InlineFeasibilityResult { claim: String, method: FeasibilityMethod,
      outcome: FeasibilityOutcome }
    GrcEngine trait additions:
      fn generate_csq(&self, function_id: &str, sensitive_ops: &[String])
        -> Result<CsqResult, GrcError>
      fn check_inline_feasibility(&mut self, claim: &str, sources: &[String])
        -> Result<InlineFeasibilityResult, GrcError>
    BasicGrcEngine struct field addition: inline_feasibility_state: InlineFeasibilityState

NEW SQLITE SCHEMAS
  None. Phase 2 introduces no DB schemas. All state held in-memory in BasicGrcEngine,
  serialized configs read from elai_cc.toml (TOML, not SQLite).

CONFIG KEYS (elai_cc.toml paths + defaults)
  Phase 2A:
    Default budget: max_calls=5, max_chars=800 (reasoning.txt s3 "moderate")
  Phase 2C TOML layering:
    [grc.defaults]
      budget = { max_calls = 5, max_chars = 800 }
      grounding_level = "high"
      csq_required = false
      allowed_methods = ["FileCheck", "SourceCheck", "WebCheck", "ApiCheck",
                         "ApiRegistryCheck", "TestCheck"]
    [agents.executor.grc]            budget=(8, 1200)
    [agents.auditor.grc]             budget=(10, 1200), required_steps=
                                     [Observe,Hypothesize,Verify,Verify],
                                     transition_overrides={Verify:[Verify, Observe]},
                                     csq_required=true, grounding_level=Verified
    [agents.context_compiler.grc]    required_steps=[Observe], allowed_methods=
                                     [FileCheck, SourceCheck]
    [agents.prompt_engineer.grc]     budget=(5, 800), required_steps=[Observe,
                                     Hypothesize, Converge]
    [agents.memory_writer.grc]       budget=(2, 400), require_scope_enumeration=false
    [agents.designer.grc]            budget=(8, 1200), required_steps=[Observe,
                                     Hypothesize, PlanAction, Verify, Converge],
                                     require_scope_enumeration=true
    NOTE: TOML keys MUST use UNDERSCORES (context_compiler, prompt_engineer,
    memory_writer) per prompt-architecture.txt L74-82, NOT hyphens.
  Phase 2D inline feasibility:
    InlineFeasibilityState.max_checks = 2 (HARDCODED, not configurable from TOML).

TESTS ADDED (counts and categories)
  Phase 2A: 12 tests in reasoning/tests/
    Categories: grounding required (T1, T6), budget overflow / forced convergence
    (T2, T5, T7), per-topic isolation (T3), invalid transition (T4), default budget
    (T8), first-step rule (T9), scope enumeration (T10, T11, T12).
  Phase 2B: 16 tests in reasoning/tests/verification_tests.rs
    Categories: FILE_CHECK pos/neg (T1, T2), SOURCE_CHECK pos/neg (T3, T4), TEST_CHECK
    (T5), network stub error (T6, T7, T8), plan sweep (T9, T10, T11), uncertain claim
    auto-resolution (T12), VerificationMethod metadata (T13), trait object safety (T14),
    adversarial regex (T15), prefix stripping (T16).
  Phase 2C: 15 tests in reasoning/tests/agent_config.rs
    Categories: per-role transition rules (T1, T2, T11), per-role budgets (T3, T4, T12,
    T13), config validation (T5, T6), anti-rubber-stamp (T7), grounding level enforcement
    (T8), allowed_methods filter (T9), defaults+overrides merge (T10), Designer
    distinctness (T14), require_scope_enumeration per role (T15).
  Phase 2D: 10 tests in reasoning/tests/csq_tests.rs
    Categories: CSQ generation (T1, T5), CSQ severity-guard logic (T2, T3, T4),
    inline feasibility outcomes Pass/Fail/Blocked (T6, T7, T9), max-2 budget (T8),
    state reset (T10).
  TOTAL: 12 + 16 + 15 + 10 = 53 phase-2 tests added.

PER-AGENT GRC BUDGETS (from Phase 2C tests T3, T4, T12, T13)
  executor:        (8, 1200)
  auditor:         (10, 1200)
  context_compiler: defaults (no explicit; required_steps=[Observe])
  prompt_engineer: (5, 800)
  memory_writer:   (2, 400)
  designer:        (8, 1200)
  Default (no agent config): (5, 800)

VERIFICATION METHODS (Phase 2B)
  FILE_CHECK            (FileCheck) — implemented (LocalVerifier via tools::read_file)
  SOURCE_CHECK          (SourceCheck) — implemented (LocalVerifier read_file + regex
                          r"(fn|struct|enum|trait|impl|const|static|type|mod)\s+
                          {symbol_name}\b")
  TEST_CHECK            (TestCheck) — implemented (CargoTestRunner via
                          BashSandbox::execute "cargo test --exact {name} -p {crate}",
                          30s timeout)
  WEB_CHECK             (WebCheck) — STUB (returns VerificationError; deferred to 4A
                          IPC proxy)
  API_CHECK             (ApiCheck) — STUB
  API_REGISTRY_CHECK    (ApiRegistryCheck) — STUB
  Methods classified per role via AgentGrcConfig.allowed_methods (Phase 2C POLICY layer);
  capability via orchestrator IPC proxy (Phase 4A CAPABILITY layer).
  Phase 2C dispatch from grounding format prefix:
    "file:..." -> FileCheck       "grep:..." -> FileCheck
    "test:..." -> TestCheck       "search:..." -> WebCheck
    "tool:..." -> SourceCheck     "training:..." -> no verification (UNCERTAIN)

CSQ SEVERITY RULES (Phase 2D D2 + I1)
  CRITICAL (guard MANDATORY): data_loss, security_breach, financial_error, auth_bypass.
    Pattern keywords (lowercase): "fs::", "file", "write", "delete", "remove", "net::",
    "tcpstream", "money", "payment", "auth", "token", "credential", "password".
    No guard => CsqResult.passed() == false => blocks AUDITING.
  WARNING (guard RECOMMENDED): incorrect_output, performance_degradation, stale_cache.
    Pattern keywords: everything else in sensitive_ops list.
    No guard => warning logged but does NOT block.
  CsqResult.passed() = checks.iter().all(|c| c.severity != Critical || c.guard_added).

REPLANREQUEST SEMANTICS (Phase 2D D4)
  PASS: claim grounded, executor proceeds with EXECUTING.
  FAIL: executor must use plan's original approach (no deviation).
  BLOCKED (fundamentally different approach detected): executor emits ReplanRequest(MAJOR)
    => triggers EXECUTING->PLANNING transition requiring human approval in BOTH
    AUTONOMOUS and INTERACTIVE modes (orchestrator.txt s8).
  Phase 2D itself only RETURNS the FeasibilityOutcome::Blocked outcome; emission of
  ReplanRequest is orchestrator's responsibility (3A+).
  Detection in Phase 2D: heuristic via CONTRADICTS_PLAN: prefix in claim string
  (placeholder; full semantic contradiction detection deferred to phase 4F
  FEASIBILITY_PROBE).

INLINE FEASIBILITY OUTCOMES (Phase 2D)
  Pass: claim found in source list (substring or exact match) — proceed.
  Fail: claim not found in sources — use plan's original approach.
  Blocked: claim contains "CONTRADICTS_PLAN:" prefix — emit ReplanRequest(MAJOR).
  Budget: max 2 inline checks per state instance (InlineFeasibilityState.max_checks=2,
  hardcoded). Third call returns Err(GrcError::BudgetExhausted) with
  topic="inline_feasibility". Counter is SEPARATE from per-topic GRC budget (cross-topic
  scope). Reset between phases via InlineFeasibilityState.reset().

DEPENDENCIES BETWEEN PHASES / SUB-STEPS
  Phase 2A -> Phase 0 (PRE_GATE: cargo check clean, contract tests pass).
  Phase 2A.2 -> 2A.1; 2A.3 -> 2A.2; 2A.4 -> 2A.3; 2A.5 -> 2A.4 (linear).
  Phase 2B -> Phase 2A complete (all 12 tests pass).
  Phase 2B introduces BREAKING CHANGES to GrcEngine trait stubs from 2A (D6 ground_plan
    return type, D7 verify_claim arity).
  Phase 2B.1 references 2A.1 stub layout; PRE-CHECK greps 2A tests for GroundingResult
    construction.
  Phase 2C -> Phase 2B complete (verification methods + plan sweep).
  Phase 2C requires 2A's GroundingLevel enum order to be CORRECTED (D5 / I3 / spec gap
    #4): 2A declared { Verified, High, Uncertain }; 2C needs { Uncertain, High, Verified }
    so derived PartialOrd works correctly. This is a retroactive 2A fix dependency.
  Phase 2C transition_overrides PRE_CHECK: 2A's allowed_next() base must be present.
  Phase 2D -> Phase 2C complete (csq_required field on AgentGrcConfig must exist).
  Phase 2D 4A forward dep: WEB_CHECK / API_CHECK stubs become real via IPC proxy.
  Phase 2D 4F forward dep: full semantic contradiction detection replaces
    CONTRADICTS_PLAN: heuristic.
  Phase 4A retroactive: agent-side budget enforcement (2C I6) demoted to advisory; orch
    becomes authoritative.
  Phase 1A DKG forward dep: provides sensitive function signatures to Phase 2D CSQ
    detection (caller passes list).
  Phase 3A+ consumes: all GRC types, the GrcEngine trait, AgentGrcConfig, CSQ outcomes.

DEFERRED / TODO / STUB MARKERS
  Phase 2A.1: VerificationResult and GroundingResult stub types created with
    #[allow(dead_code)] for forward use in 2B. verify_claim and ground_plan trait
    methods are unimplemented!() stubs marked #[allow(unused)] until 2B.
  Phase 2B.1: NetworkVerifier (WEB_CHECK, API_CHECK, API_REGISTRY_CHECK) stubbed —
    real impl deferred to provider integration / 4A IPC proxy.
  Phase 2B.4 I2: WEB_CHECK, API_CHECK, API_REGISTRY_CHECK return
    VerificationError("network verification not implemented in phase 2B"). This is
    intentional and not a pass-through — the dispatch + trait are complete.
  Phase 2C.4 I6: agent-side budget enforcement is "AUTHORITATIVE for phases 0-3
    (in-process)"; demoted to advisory in 4A.
  Phase 2D.4 I2: Blocked detection uses CONTRADICTS_PLAN: prefix as a placeholder.
    Full semantic contradiction detection deferred to Phase 4F (FEASIBILITY_PROBE).
  Phase 2D D5: FeasibilityMethod enum has only SourceMatch; future variants WebSearch,
    Grep, ApiRegistryCheck added in phase 4F.
  Spec drift gaps (Phase 2C): 5 spec items in reasoning.txt require updates after
    s-1003192 releases reasoning.txt (see SPEC GAPS section).

================================================================
## PHASE 2 INTERFACE MAP
================================================================

### Phase 2A — public Rust API surface
  Located: .elai_cc/crates/reasoning/src/{lib.rs, types.rs, engine.rs}
  Public types (re-exported from lib.rs):
    GrcState, ThinkStep, ThinkResult, GrcBudget, GroundingLevel, GrcError,
    BasicGrcEngine.
  Public trait:
    GrcEngine — 6 methods total:
      think(&mut self, ThinkStep) -> Result<ThinkResult, GrcError>     [implemented 2A]
      set_budget(&mut self, &str, u32, u32)                            [implemented 2A]
      get_budget(&self, &str) -> Option<&GrcBudget>                    [implemented 2A]
      reset_topic(&mut self, &str)                                     [implemented 2A]
      verify_claim(&self, &str, &str) -> Result<...>                   [stub for 2B]
      ground_plan(&self, &str) -> Vec<GroundingResult>                 [stub for 2B]
  reasoning crate types exposed: as above; plus stubs VerificationResult,
    GroundingResult (for 2B forward refs).
  Integration points where downstream phases (3A+) consume:
    - Phase 3A orchestrator reads ThinkStep via IPC, validates via GrcEngine::think.
    - Phase 3+ all agents thread GrcBudget + GrcState chain through topic-keyed call sites.
    - GroundingLevel + GrcError are returned to orchestrator FSM transitions for
      block/proceed decisions.
    - workflow plan files (PLANNING -> EXECUTING) inspect think() outputs to gate moves.

### Phase 2B — public Rust API surface
  Located: + .elai_cc/crates/reasoning/src/verification.rs (new)
  Public types added:
    VerificationMethod (6 variants), PlanGroundingResult, TestRunResult.
    Updated VerificationResult, GroundingResult (typed method + verified_at).
    GrcError variant: VerificationError.
  Public traits added:
    Verifier: Send + Sync (object-safe; carries verify + supported_methods).
    TestRunner: Send + Sync.
  Public impls added:
    LocalVerifier (FILE_CHECK, SOURCE_CHECK, TEST_CHECK).
    NetworkVerifier (WEB_CHECK, API_CHECK, API_REGISTRY_CHECK — stubs returning
    VerificationError).
    CargoTestRunner, MockTestRunner.
  Trait signature changes (BREAKING from 2A):
    fn verify_claim(&self, &str, &str, &str) -> Result<VerificationResult, GrcError>
    fn ground_plan(&self, &str) -> PlanGroundingResult
  reasoning crate types exposed: as above.
  Integration points where downstream phases (3A+) consume:
    - Phase 3A pre-EXECUTING hook calls GrcEngine::ground_plan(plan_text); blocks the
      transition when PlanGroundingResult.all_grounded == false.
    - Phase 4A wraps Verifier behind IPC GroundingRequest message; reasoning crate's
      Verifier abstraction is preserved (Send + Sync, &self for stateless verify_claim).
    - Phase 4A converts verify_claim into IPC dispatch using VerificationMethod enum
      for routing (agent role -> allowed methods -> orchestrator capability).

### Phase 2C — public Rust API surface
  Located: + .elai_cc/crates/reasoning/src/config.rs (new)
  Public types added:
    AgentGrcConfig (8 fields), GrcDefaults.
    GrcError variants: InsufficientFindings, ConfigValidation.
    Constant: FINDING_PREFIX = "FINDING:".
  Public traits added:
    AgentGrcConfigLoader { load_config, validate_config }.
  Public impls added:
    TomlConfigLoader (reads via core::Config; merges [grc.defaults] +
    [agents.{role}.grc]).
  Trait method additions to GrcEngine:
    fn with_agent_config(&mut self, AgentGrcConfig)
    fn get_agent_config(&self) -> Option<&AgentGrcConfig>
  Effective behavior changes:
    GroundingLevel: PartialOrd (Uncertain < High < Verified).
    Engine: per-step routing applies budget, transition_overrides (additive),
    required_steps whitelist, grounding_level minimum, allowed_methods filter,
    require_scope_enumeration WARN, anti-rubber-stamp (auditor: Verify with FINDING:
    prefix required before Converge).
  reasoning crate types exposed: as above; AgentRole stays as String to avoid cross-crate
    enum coupling.
  Integration points where downstream phases (3A+) consume:
    - Phase 3A orchestrator: spawns each agent with role -> loads AgentGrcConfig via
      TomlConfigLoader -> calls GrcEngine::with_agent_config before any agent think()
      calls.
    - Phase 3A FSM: orchestrator inspects get_agent_config() to surface budgets/policies
      to dashboard.
    - Phase 4A IPC: orchestrator enforces budget externally per L440-444; agent-side
      becomes advisory. AgentGrcConfig fields cross IPC boundary at agent spawn time.
    - Phase 4A also enforces allowed_methods on GroundingRequest IPC messages (second
      layer of policy enforcement).
    - Memory-bank / dashboard read get_agent_config() for diagnostics.

### Phase 2D — public Rust API surface
  Located: + extensions to .elai_cc/crates/reasoning/src/{types.rs, engine.rs}
  Public types added:
    CsqSeverity (Critical, Warning), CsqCheck, CsqResult (with computed passed()),
    InlineFeasibilityState (max_checks=2 default), FeasibilityOutcome (Pass, Fail,
    Blocked), FeasibilityMethod (SourceMatch only in 2D), InlineFeasibilityResult.
  Trait method additions to GrcEngine:
    fn generate_csq(&self, &str, &[String]) -> Result<CsqResult, GrcError>
    fn check_inline_feasibility(&mut self, &str, &[String])
      -> Result<InlineFeasibilityResult, GrcError>
  Effective behavior: BasicGrcEngine carries InlineFeasibilityState; CSQ severity
    classifier uses keyword pattern list (lowercase comparison); inline feasibility
    Blocked detection limited to CONTRADICTS_PLAN: prefix heuristic.
  reasoning crate types exposed: as above. Phase 2 exit means 7 new 2D types + the
    Phase 2A/B/C types are all public.
  Integration points where downstream phases (3A+) consume:
    - Phase 3A executor pipeline: before AUDITING, calls generate_csq for each modified
      function; if CsqResult.passed() == false, blocks AUDITING transition until guards
      added.
    - Phase 3A executor pipeline: during EXECUTING, when verify detects new claim,
      calls check_inline_feasibility(claim, sources); routes outcome:
        Pass -> continue
        Fail -> revert to plan's original approach
        Blocked -> orchestrator emits ReplanRequest(MAJOR), forces EXECUTING->PLANNING,
          requires human approval (orchestrator.txt s8).
    - Phase 1A DKG: provides sensitive_ops list (functions touching files/network/money/
      auth) to caller, which passes them to generate_csq.
    - Phase 4F (FEASIBILITY_PROBE): replaces CONTRADICTS_PLAN heuristic with full
      semantic contradiction detection; extends FeasibilityMethod with WebSearch, Grep,
      ApiRegistryCheck variants.
    - Memory-bank / dashboard surfaces CsqResult / InlineFeasibilityResult for review.

END OF PHASE 2 LOSSLESS SUMMARY
