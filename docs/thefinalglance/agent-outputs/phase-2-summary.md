PHASE 2: REASONING ENGINE (LOSSLESS SUMMARY)

Source files:
- F:\.ELAI_workflow\docs\specs\guide\steps\phase-2A.txt (24,128 bytes)
- F:\.ELAI_workflow\docs\specs\guide\steps\phase-2B.txt (37,770 bytes)
- F:\.ELAI_workflow\docs\specs\guide\steps\phase-2C.txt (33,046 bytes)
- F:\.ELAI_workflow\docs\specs\guide\steps\phase-2D.txt (28,356 bytes)

Phase 2 overall scope: GRC state machine (2A) + grounding verification (2B) +
per-agent GRC config (2C) + CSQ + inline feasibility (2D). After 2D complete,
reasoning/ crate is feature-complete.


==============================================================
PHASE 2A: GRC STATE MACHINE (basic)
==============================================================

Prerequisites: Phase 0 complete (all contract tests pass, cargo check clean).

GOAL
Implement Grounded Reasoning Chain engine core. All reasoning cites grounding sources.
Budget system enforces per-topic call + character limits. State machine validates
transition order. Rejected steps produce typed errors.

Scope restricted to GRC state machine, budget system, grounding FORMAT validation only.
Actual grounding verification methods (FILE_CHECK, WEB_CHECK, etc.) -> Phase 2B.
Per-agent config -> Phase 2C. CSQ -> Phase 2D.

READ BEFORE IMPLEMENTING
  docs/specs/crates/reasoning.txt Sections 1-3 (responsibilities, GRC state machine, budget)
  docs/specs/crates/reasoning.txt Section 10 (KEY TYPES: ThinkStep, GrcBudget, GrcError, etc.)
  docs/specs/crates/reasoning.txt Section 11 (TESTABLE INVARIANTS: items 1-3 are phase-2A scope)
  docs/specs/architecture-overview.txt Section 4 (GrcEngine trait: think, set_budget)
  docs/specs/integration/prompt-architecture.txt Section 1 (why code-driven, not prompt-driven)

DESIGN DECISIONS (grounded)
  D1. State transition validation via allowed_transitions map.
      GRC state machine (observe -> hypothesize -> plan_action -> act -> verify) rejects
      invalid transitions (e.g., hypothesize -> verify skipping plan_action).
      GrcState enum with fn allowed_next(&self) -> &[GrcState].
      Grounding: standard FSM. guidance-ai uses FSM-from-schema token masking; our version
      validates at step level.
  D2. Typed error enum for all rejection modes.
      GrcError { UngroundedStep, BudgetExhausted, InvalidTransition, MalformedGrounding }.
      Each variant carries context (step, what was missing, what was expected).
      Grounding: thiserror pattern.
  D3. Budget tracking with forced convergence.
      GrcBudget tracks remaining_calls + remaining_chars, decremented atomically per step.
      When remaining_calls <= 1, engine forces convergence step.
      Grounding: BudgetThinker (OpenReview 2025, openreview.net/forum?id=ahatk5qrmB) inserts
      budget-remaining signals; Token-Budget-Aware LLM Reasoning (ACL 2025) adjusts tokens
      by complexity. Our version: hard limits at engine level.
  D4. Grounding field validation decomposes content into claims.
      Instead of "grounding field present," parse grounding string into individual source
      citations, verify each well-formed (format "source_type:reference", e.g.,
      "file:src/foo.rs:L42"). Phase 2B adds actual verification; 2A validates format only.
      Grounding: RARR (arxiv:2210.08726) decomposes LLM output; SAFE (arxiv:2403.18802) similar.
  D5. GroundingLevel enum matching spec hierarchy.
      VERIFIED > HIGH > UNCERTAIN. Steps with UNCERTAIN grounding flagged (warn log) but
      not rejected in 2A. Phase 2B adds resolution.
      Grounding: reasoning.txt section 4.


STEPS

2A.1 SKELETON: reasoning/ crate structure + types
  PRE_GATE:
    - cargo check succeeds (Phase 0 baseline clean)
    - reasoning/ crate does NOT exist yet (greenfield)
  TASK:
    - Create .elai_cc/crates/reasoning/ with Cargo.toml (deps: core/)
    - Add reasoning crate to workspace Cargo.toml
    - Define types in reasoning/src/types.rs:

        GrcState enum { Observe, Hypothesize, PlanAction, Act, Verify, Converge, Status }
          #[derive(Debug, Clone, Copy, PartialEq, Eq)]
          fn allowed_next(&self) -> &[GrcState]
          Note: Converge allowed from any state. Status allowed from any state.
          First non-Status/non-Converge step in topic must be Observe.

        ThinkStep {
          step_type: GrcState,
          content: String,
          topic: String,
          grounding: Option<String>,
          conclusion: Option<String>,
          scope: Option<Vec<String>>
        }
          #[derive(Debug, Clone)]
          scope: when AgentGrcConfig.require_scope_enumeration=true, the Observe step
          must populate scope with all task components/modalities. Orchestrator validates
          every scope item appears in subsequent plan_action. Unaddressed -> WARN in agent context.
          Source: reasoning.txt section 10, technical-analysis-v3.txt Concern 3.

        ThinkResult {
          step: ThinkStep,
          budget_remaining: GrcBudget,
          grounding_level: Option<GroundingLevel>,
          warnings: Vec<String>
        }
          #[derive(Debug, Clone)]

        GrcBudget {
          max_calls: u32,
          max_chars: u32,
          calls_used: u32,
          chars_used: u32
        }
          #[derive(Debug, Clone, Copy, PartialEq, Eq)]
          fn remaining_calls(&self) -> u32
          fn remaining_chars(&self) -> u32
          fn is_exhausted(&self) -> bool
          fn must_converge(&self) -> bool (remaining_calls <= 1)
          Default: max_calls=5, max_chars=800 (reasoning.txt section 3 "moderate" default)

        GroundingLevel enum { Verified, High, Uncertain }
          #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]

        GrcError enum {
          UngroundedStep { step_type, topic },
          BudgetExhausted { topic, calls_used, max_calls },
          InvalidTransition { from: GrcState, to: GrcState, allowed: Vec<GrcState> },
          MalformedGrounding { raw, reason }
        }
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
        Rationale: defining full 6-method trait now prevents breaking BasicGrcEngine impls
        when 2B adds verify_claim/ground_plan. Stubs use unimplemented!() with #[allow(unused)].

    - Define stub types for 2B forward references:
        VerificationResult { claim: String, passes: bool, evidence: String, method_used: String }
        GroundingResult { claim: String, method: String, level: GroundingLevel, source: String }
        Both marked #[allow(dead_code)] until 2B.

    - Stub implementation: BasicGrcEngine with HashMap<String, TopicState>
        TopicState {
          budget: GrcBudget,
          last_state: Option<GrcState>,
          steps: Vec<ThinkStep>,
          grounding_log: Vec<(String, GroundingLevel)>
        }
        Default budget applied when think() called on topic with no prior set_budget().

    - Re-export public API from reasoning/src/lib.rs

  DETERMINISTIC_CHECK:
    - cargo check on workspace: 0 errors
    - All types exist in source (grep confirms each type appears once)
    - GrcState derives Clone (required for InvalidTransition variant)

  ADVERSARIAL:
    - If GrcState::allowed_next() wrong (e.g., allows Observe->Verify), invalid chains pass silently.
      Test: Observe.allowed_next() does NOT contain Verify.
    - If GrcBudget uses signed integers, underflow possible.
      Test: remaining_calls() returns 0 when calls_used >= max_calls, never wraps to u32::MAX.

  ACCEPTANCE:
    - cargo check succeeds with reasoning/ in workspace: exit 0
    - GrcEngine trait has exactly 6 methods (4 implemented + 2 phase-2B stubs)

2A.2 VERIFY: compilation check
  PRE_GATE:
    - 2A.1 complete (reasoning/ exists and compiles)
  TASK:
    - cargo check on full workspace
    - Verify reasoning/ depends ONLY on core/ (no other crate deps for 2A)
  DETERMINISTIC_CHECK:
    - cargo check: 0 errors
    - deps: only core, thiserror, serde (nothing else)
  ADVERSARIAL:
    - reasoning/ accidentally depends on providers/ or tools/ creates premature coupling.
      Phase 2A is pure logic; no LLM calls.
  ACCEPTANCE:
    - exit 0
    - deps only core/, thiserror, serde (no providers/, tools/, knowledge/)

2A.3 TEST: write failing tests (TDD) — 12 tests
  PRE_GATE: 2A.2 passes; GrcEngine trait exists.
  TASK: Write tests in reasoning/tests/ (all must FAIL against stub):

    T1. observe_requires_grounding:
        think(Observe, grounding:None) -> Err(UngroundedStep).
        Invariant: reasoning.txt s11 item 1.

    T2. budget_overflow_forces_convergence:
        set_budget(topic="t", max_calls=3, max_chars=2000).
        Call think() twice (non-Converge, uses 2 calls, remaining=1).
        must_converge()==true. think(non-Converge) -> Err(BudgetExhausted).
        think(Converge) succeeds (uses call 3). Next think() -> Err(BudgetExhausted).
        Note: max_calls=3 (not 2) because must_converge fires at remaining<=1.
        Invariant: reasoning.txt s11 item 2.

    T3. per_topic_budget_independence:
        set_budget(topic="a", max_calls=2). set_budget(topic="b", max_calls=3).
        Exhaust "a". Assert "b" still has remaining_calls == 3.

    T4. invalid_transition_rejected:
        think(Observe), think(Verify) (skip Hypothesize, PlanAction).
        -> Err(InvalidTransition{from:Observe, to:Verify, allowed:[Hypothesize]}).

    T5. must_converge_at_budget_limit:
        set_budget(max_calls=3). Use 2 calls. must_converge() == true.
        Next think() must be Converge or rejected.

    T6. malformed_grounding_rejected:
        think(Observe, grounding:Some("no_colon_no_type")) -> Err(MalformedGrounding).

    T7. status_step_free (does not consume budget):
        set_budget(max_calls=2). think(Status). calls_used == 0.

    T8. default_budget_applied_on_first_think:
        No set_budget(). think(Observe) on new topic succeeds (default max_calls=5, max_chars=800).
        get_budget(topic).unwrap().max_calls == 5.

    T9. first_step_must_be_observe:
        think(Hypothesize) as first step on new topic -> Err(InvalidTransition) with from=None.
        Exception: Status and Converge allowed as first step.

    T10. scope_enumeration_present_when_required:
         AgentGrcConfig.require_scope_enumeration=true.
         think(Observe, scope=Some(vec!["ui", "api", "tests"])) succeeds.
         think(PlanAction) addressing only "ui", "api" -> ThinkResult.warnings has WARN for "tests".
         Invariant: reasoning.txt s10, technical-analysis-v3.txt Concern 3.

    T11. scope_enumeration_missing_when_required:
         AgentGrcConfig.require_scope_enumeration=true.
         think(Observe, scope=None) -> Err or ThinkResult with warning.

    T12. scope_enumeration_optional_when_not_required:
         require_scope_enumeration=false.
         think(Observe, scope=None) succeeds with no scope-related warnings.

  DETERMINISTIC_CHECK:
    - >=12 tests exist and FAIL (TDD)
    - compile errors expected (stubs use todo!())
    - test names pattern match: grep -c "#\[test\]" reasoning/tests/*.rs >= 12

  ADVERSARIAL:
    - Tests pass against stubs = stubs too permissive. Stubs must use todo!()/unimplemented!().
    - Budget test uses sleep or timing = non-deterministic. grep "sleep\|Duration\|Instant" returns 0.

  ACCEPTANCE:
    - >= 12 test functions exist
    - all tests fail to compile or fail at runtime (TDD)
    - zero timing-dependent tests
    - T10-T12 (scope enumeration) present and fail against stubs

2A.4 IMPLEMENT: GRC state machine logic
  PRE_GATE: 2A.3 complete; mcp_session_claim at EXECUTING entry.
  TASK: Implement BasicGrcEngine:

    I1. State transition validation:
        Maintain last_state per topic. On think(), check step.step_type in
        last_state.allowed_next(). If not, return Err(InvalidTransition).
        Special cases:
          First step in topic: must be Observe (no last_state).
          Status step: allowed from any state, does not update last_state.
          Converge step: allowed from any state (forced convergence path).

    I2. Grounding validation:
        For steps requiring grounding (Observe, Hypothesize, Verify):
          If grounding is None: return Err(UngroundedStep).
          If grounding is Some(s): parse as comma-separated "type:reference" pairs.
          Valid types: "file", "grep", "test", "search", "tool", "training".
          Malformed -> Err(MalformedGrounding).
          Classify each citation's GroundingLevel:
            "test"/"search" -> Verified
            "file"/"grep"/"tool" -> High
            "training" -> Uncertain
          Rationale: "search" = web-verified (reasoning.txt s4). "tool" = tool output read
          this session (equivalent to file/grep = HIGH).
          Log warning for Uncertain (do not reject in 2A).
        For PlanAction, Act: grounding optional (action steps).
        For Status: no grounding required, no budget consumed.

    I3. Budget enforcement (check-then-increment pattern):
        On each think() except Status:
          CHECK FIRST: if calls_used >= max_calls -> Err(BudgetExhausted) immediately.
          CHECK FIRST: if chars_used + step.content.len() > max_chars -> Err(BudgetExhausted).
          CHECK FIRST: if must_converge() (remaining_calls <= 1) and step_type != Converge
            -> Err(BudgetExhausted) with forced-convergence message.
          THEN INCREMENT: calls_used += 1, chars_used += step.content.len()
            (saturating_add to prevent u32 overflow).
        Rationale: increment-before-check corrupts budget state on error path.
        After BudgetExhausted, calls_used would be max_calls+1, making subsequent
        budget queries incorrect.

    I4. Topic isolation:
        Each topic has independent TopicState { budget, last_state, steps, grounding_log }.
        Default budget from global config (configurable via set_budget).
        reset_topic() clears all state for topic.

    I5. ThinkResult construction:
        Return ThinkResult { step, budget_remaining, grounding_level, warnings }.

  DETERMINISTIC_CHECK:
    - All 12 tests pass (0 failed)
    - No todo!()/unimplemented!() in engine.rs
    - No unwrap() in engine.rs (all errors typed)

  ADVERSARIAL:
    - allowed_next() returns empty for Verify (terminal) = valid chains cannot complete.
      Verify.allowed_next() must contain at least Observe (new chain start) or be explicitly terminal.
    - chars_used overflows u32 at ~4GB = budget tracking breaks. Check before increment (saturating_add or checked_add).
    - Topic HashMap grows unbounded = memory leak. After reset_topic(), entry fully removed from map (not zeroed).

  ACCEPTANCE:
    - cargo test -p reasoning: 0 failures, >= 12 tests pass
    - cargo check --workspace: 0 errors
    - grep todo!/unimplemented!/unwrap() in engine.rs: 0

2A.5 MEASURE: performance and correctness metrics
  PRE_GATE: 2A.4 complete.
  TASK:
    - Measure per-step validation overhead (target <5ms, pure logic no I/O)
    - Measure memory per topic (target <1KB base + step history)
    - Record: test count, LOC, compile time delta from Phase 0 baseline
    - Write .elai_cc/metrics/phase_2A.toml
  DETERMINISTIC_CHECK:
    - phase_2A.toml exists and contains validation_overhead_us
    - validation_overhead_us < 5000
  ADVERSARIAL:
    - Wall-clock time in CI = non-deterministic. std::hint::black_box + std::time::Instant for local benchmarks only. Report median of 1000 iterations. Do NOT add criterion without user approval.
    - Instant for benchmarks acceptable under entropy quarantine (measurement metadata, not deterministic output).
  ACCEPTANCE:
    - .elai_cc/metrics/phase_2A.toml exists with all fields
    - validation_overhead_us < 5000
    - cargo test: 0 failures

PHASE 2A EXIT CRITERIA
  GRC enforces grounding on every step requiring it (Observe, Hypothesize, Verify).
  Budget overflow triggers convergence or returns BudgetExhausted.
  Invalid transitions rejected with InvalidTransition.
  Malformed grounding strings rejected with MalformedGrounding.
  Status steps free (no budget consumed, no grounding required).
  Per-topic budgets independent.
  Scope enumeration validated when require_scope_enumeration=true (WARN on unaddressed).
  All 12+ tests pass. Zero todo!/unimplemented!/unwrap() in engine code.

PHASE 2A GROUNDING
  "GRC state machine: observe->hypothesize->plan_action->act->verify" -> GROUNDED reasoning.txt s2
  "Budget per-topic: max_calls 1-12, max_chars 200-1500" -> GROUNDED reasoning.txt s3
  "Grounding required on every step" -> GROUNDED reasoning.txt s2, s11 item 1
  "Budget overflow forces convergence" -> GROUNDED reasoning.txt s3, s11 item 2
  "Status step is free echo" -> VERIFIED (CLAUDE.md GRC tools section)
  "<5ms validation overhead" -> UNGROUNDED (verified empirically in 2A.5)
  "Forced convergence at remaining_calls<=1" -> GROUNDED BudgetThinker (OpenReview 2025). Our version simpler (hard limit vs control tokens).
  "Claim decomposition in grounding validation" -> GROUNDED RARR (arxiv:2210.08726), SAFE (arxiv:2403.18802). Our version validates format (2A); actual verification 2B.
  "Hard/soft grounding: reject UngroundedStep but warn on Uncertain" -> GROUNDED DSPy Assertions (arxiv:2312.13382). Our GroundingLevel maps: VERIFIED/HIGH = proceed, UNCERTAIN = warn.
  "State transition validation via allowed_transitions" -> GROUNDED standard FSM. guidance-ai FSM token masking; our version validates at step level.
  "thiserror for GrcError" -> VERIFIED thiserror 2.x (crates.io 290M+ downloads)
  "Memory per topic <1KB base" -> UNGROUNDED (verify in 2A.5)
  "ThinkStep.scope: Option<Vec<String>>" -> GROUNDED reasoning.txt s10, technical-analysis-v3.txt Concern 3
  "Scope enumeration validated when require_scope_enumeration=true" -> GROUNDED technical-analysis-v3.txt testable invariants


==============================================================
PHASE 2B: GROUNDING VERIFICATION
==============================================================

Prerequisites: Phase 2A complete (all 12+ tests pass, 0 failures).

GOAL
Implement verification methods that check claims against real sources before execution.
2A validates grounding FORMAT (string parsing). 2B resolves grounding claims by actually
checking files, source code, tests, web, APIs. Ungrounded claims in a plan block the
PLANNING -> EXECUTING FSM transition.

6 verification methods: FILE_CHECK, SOURCE_CHECK, TEST_CHECK, WEB_CHECK, API_CHECK,
API_REGISTRY_CHECK. Each returns binary pass/fail with evidence string. No confidence scores.

Plan grounding sweep: before EXECUTING, GRC scans all claims in plan, classifies each
as GROUNDED / VERIFY_NEEDED / UNGROUNDED, runs verification on VERIFY_NEEDED, blocks
execution if any remain UNGROUNDED.

Phase 2B also resolves UNCERTAIN from 2A. Steps with Uncertain that can be resolved via
FILE_CHECK/SOURCE_CHECK are auto-promoted to Verified/High. Unresolvable remain UNCERTAIN
(flagged but not auto-rejected — per-agent policy in 2C).

FORWARD DEPENDENCY: Phase 4A changes verification execution model — agents proxy
verification via IPC GroundingRequest messages through orchestrator. 2B implements
as direct function calls; 4A wraps them behind IPC.

READ BEFORE IMPLEMENTING
  docs/specs/crates/reasoning.txt Section 4 (grounding verification, 6 methods)
  docs/specs/crates/reasoning.txt Section 5 (fact-based planning protocol)
  docs/specs/crates/reasoning.txt Section 8 (inline feasibility — 2D, context)
  docs/specs/crates/reasoning.txt Section 9 (process isolation — forward on 4A)
  docs/specs/crates/reasoning.txt Section 10 (KEY TYPES: GroundingResult, VerificationResult)
  docs/specs/crates/reasoning.txt Section 11 (TESTABLE INVARIANTS item [2B])
  docs/specs/architecture-overview.txt Section 4 (verify_claim [2B stub], ground_plan [2B stub])
  docs/specs/guide/steps/phase-2A.txt Step 2A.1 (stubs this phase replaces)
  docs/specs/crates/tools.txt (FILE_CHECK/SOURCE_CHECK use tools/ crate)

DESIGN DECISIONS (grounded)
  D1. Binary verification, not scalar confidence.
      All 6 methods: VerificationResult { passes: bool, evidence: String }.
      FILE_CHECK inherently binary. SOURCE_CHECK binary (signature matches or not). TEST_CHECK binary.
      WEB_CHECK/API_CHECK could theoretically return scores, but we keep binary for uniformity — evidence string carries detail.
      Grounding: MiniCheck (Tang et al., EMNLP 2024, arxiv:2404.10774) shows claim-level fact-checking works well with binary labels. Google Vertex AI grounding uses binary grounded/ungrounded in fast mode.
      Deterministic benefit: compose cleanly — plan GROUNDED iff ALL claims pass. No threshold tuning.

  D2. Claim decomposition for plan sweep only.
      Individual verify_claim() checks one claim at a time. Plan sweep (ground_plan) decomposes plan text into individual claims.
      Our decomposition is STRUCTURAL (extract file paths, function names, API references), not semantic (no NLI model). Simpler + deterministic.
      Grounding: RARR (arxiv:2210.08726), SAFE (Wei et al. 2024) use LLM-based decomposition for semantic claims. Our claims are structural references ("file:src/main.rs", "api:ollama/generate") — regex extraction suffices. Phase 2A already validates "type:reference" format, so plan sweep reuses that parsed structure.

  D3. VerificationMethod enum for dispatch.
      VerificationMethod { FileCheck, SourceCheck, TestCheck, WebCheck, ApiCheck, ApiRegistryCheck }.
      verify_claim() takes method + claim + context, dispatches to correct checker.
      Enables per-method availability filtering (Phase 4A restricts which methods agents can use via IPC proxy).
      Grounding: reasoning.txt s9 L140-147 defines per-method availability by agent role.

  D4. verified_at via entropy-quarantined timestamp.
      GroundingResult includes verified_at: Timestamp. All time access through core::entropy::TimeSource (Phase 0 quarantine). Deterministic testing — test fixtures inject fixed timestamps.
      Grounding: implementation-guide.txt L46 ("all time/randomness via injectable traits"). reasoning.txt L184.

  D5. tools/ crate dependency for FILE_CHECK, SOURCE_CHECK.
      2B adds tools/ as reasoning/ dep. FILE_CHECK uses tools::read_file for existence check. SOURCE_CHECK uses tools::read_file + parsing. First reasoning/ -> tools/ coupling.
      Grounding: crate map lists reasoning/ deps as (core, providers, tools).
      Circular dep risk: reasoning -> tools -> knowledge -> (no reasoning dep, safe). knowledge/ does NOT depend on reasoning/ (crate map verified).
      Note: tools/ transitively pulls knowledge/ -> providers/. reasoning/ will eventually depend on providers/ directly. Not circular, but providers/ changes break reasoning/ through two paths.

  D6. GrcEngine trait signature update (BREAKING CHANGE from 2A stubs).
      2A: fn ground_plan(&self, plan: &str) -> Vec<GroundingResult>
      2B: fn ground_plan(&self, plan: &str) -> PlanGroundingResult
      Vec<GroundingResult> loses sweep summary. PlanGroundingResult provides answer directly (all_grounded, ungrounded, verify_needed).
      Impact: 2A stubs return unimplemented!(), no existing code constructs Vec<GroundingResult>. Safe change.
      Architecture-overview.txt and reasoning.txt must be updated.
      PRE_CHECK: grep 2A tests for "ground_plan" — if any test calls ground_plan and checks return type, must update.

  D7. verify_claim gets a context parameter.
      2A: fn verify_claim(&self, claim: &str, source: &str) -> Result<...>
      2B: fn verify_claim(&self, claim: &str, source: &str, context: &str) -> Result<...>
      Context needed for SOURCE_CHECK (claim="symbol_name", context="file_path") and TEST_CHECK (claim="test_name", context="crate_name"). FILE_CHECK uses claim as path, ignores context. WEB_CHECK/API_CHECK may use context for search scope.
      Breaking change. 2A stub unimplemented!(), no existing code calls with 2 args. Safe.

  D8. TestRunner trait abstraction for TEST_CHECK.
      TEST_CHECK runs specific test by name. Production: BashSandbox::execute("cargo test --exact {name} -p {crate}", sandbox_config).
      Unit tests: mocked (spawning cargo is too slow, non-deterministic).
      trait TestRunner: Send + Sync { fn run_test(&self, name: &str, crate_name: &str) -> TestRunResult; }
      LocalVerifier takes Box<dyn TestRunner> at construction (DI).
      Production: CargoTestRunner (BashSandbox::execute with 30s timeout).
      Test: MockTestRunner (configured pass/fail per test name).
      Grounding: tools.txt s9 L184-186 defines BashSandbox::execute(command, sandbox_config) -> BashResult.

RISKS
  R-2B-1: tools/ dep introduces coupling earlier than other phases may expect.
    Mitigation: reasoning/ uses only tools::read_file (leaf). Verify tools/Cargo.toml doesn't depend on reasoning/.
  R-2B-2: WEB_CHECK/API_CHECK require network. Phase 2B tests MUST NOT require network.
    Mitigation: all tests use mock/stub. Real network checks only in integration tests (gated by #[cfg(feature="network")]).
  R-2B-3: Plan sweep with zero claims (empty plan). Must not panic or return UNGROUNDED.
    Empty plan = trivially GROUNDED.
  R-2B-4: Phase 4A wraps verification behind IPC. 2B's direct-call interface must allow 4A wrapping without changing trait signature.
    Mitigation: verify_claim takes &self (not &mut self), stateless per call.


STEPS

2B.1 SKELETON: verification types + trait extension
  PRE_GATE:
    - Phase 2A tests pass (0 failed)
    - verify_claim and ground_plan stubs exist in engine.rs (from 2A.1)
    - VerificationResult and GroundingResult stub types exist (from 2A.1)
  TASK:
    - Add VerificationMethod enum to reasoning/src/types.rs:
        VerificationMethod { FileCheck, SourceCheck, TestCheck, WebCheck, ApiCheck, ApiRegistryCheck }
          #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
          fn requires_network(&self) -> bool (WebCheck, ApiCheck, ApiRegistryCheck = true)
          fn requires_tools(&self) -> bool (FileCheck, SourceCheck, TestCheck = true)

    - PRE-CHECK: grep 2A tests for GroundingResult construction. If any test constructs GroundingResult with method: String, must be updated to use VerificationMethod enum.

    - Update GroundingResult:
        GroundingResult {
          claim: String,
          method: VerificationMethod,
          level: GroundingLevel,
          source: String,
          verified_at: u64
        }
        verified_at: u64 unix timestamp from core::entropy::TimeSource.
        method: change from String to VerificationMethod (type-safe dispatch).

    - Update VerificationResult:
        VerificationResult {
          claim: String,
          passes: bool,
          evidence: String,
          method_used: VerificationMethod
        }
        method_used: String -> VerificationMethod.

    - Add PlanGroundingResult:
        PlanGroundingResult {
          claims: Vec<GroundingResult>,
          all_grounded: bool,
          ungrounded: Vec<String>,
          verify_needed: Vec<String>
        }
        Semantics of verify_needed at return time:
          - Contains claims type "training:" that COULD NOT be auto-resolved.
          - Claims "training:" successfully auto-resolved are in claims with level=Verified, NOT in verify_needed.
          - verify_needed items are a SUBSET of ungrounded.
          - all_grounded = ungrounded.is_empty() (verify_needed does not affect this).

    - Add VerificationError to GrcError enum:
        VerificationError { method: VerificationMethod, claim: String, reason: String }

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
        Implementations (stubs):
          CargoTestRunner { sandbox: Box<dyn BashSandbox>, timeout_ms: u64 }
          MockTestRunner { results: HashMap<String, TestRunResult> }

    - Update GrcEngine trait stubs (BREAKING — see D6, D7):
        fn verify_claim(&self, claim: &str, source: &str, context: &str) -> Result<VerificationResult, GrcError>
          (added context parameter — 2A had 2, now 3)
        fn ground_plan(&self, plan: &str) -> PlanGroundingResult
          (changed return — 2A returned Vec<GroundingResult>)
        Update BasicGrcEngine stub impls (still unimplemented!()).

    - Add tools/ dep to reasoning/Cargo.toml.

    - Stub implementations:
        LocalVerifier (FILE_CHECK, SOURCE_CHECK, TEST_CHECK) — todo!()
        NetworkVerifier (WEB_CHECK, API_CHECK, API_REGISTRY_CHECK) — todo!()
        LocalVerifier::new(verifier: Box<dyn TestRunner>) — takes TestRunner for TEST_CHECK

    - Re-export new types from reasoning/src/lib.rs

  DETERMINISTIC_CHECK:
    - cargo check: 0 errors
    - VerificationMethod enum (1 occurrence)
    - PlanGroundingResult struct (1)
    - Verifier trait in verification.rs (1)
    - VerificationMethod has exactly 6 variants (6 "Check" matches)
    - GroundingResult has verified_at
    - tools/ in reasoning/Cargo.toml
    - Phase 2A tests still pass

  ADVERSARIAL:
    - GroundingResult.method is String instead of VerificationMethod -> string matching fragile. grep "method: String" in GroundingResult = 0.
    - Verifier trait not object-safe (uses generics or Self by value) -> cannot use dyn Verifier. Add compile test: let _: Box<dyn Verifier>.
    - tools/ dep creates circular chain -> workspace won't compile. cargo check succeeds.

  ACCEPTANCE:
    - cargo check: 0 errors
    - VerificationMethod has 6 variants
    - Verifier object-safe
    - 2A tests still pass (0 regressions)

  GROUNDING:
    - "6 verification methods" -> GROUNDED reasoning.txt s4 L46-55
    - "verified_at in GroundingResult" -> GROUNDED reasoning.txt L184
    - "tools/ dependency for reasoning/" -> GROUNDED implementation-guide.txt L64 crate map
    - "Verifier trait with Send + Sync" -> VERIFIED (inference): 4A proxies verification via IPC (reasoning.txt s9 L134 "agents are isolated processes"). IPC implies cross-thread message passing, requires Send+Sync. reasoning.txt L137 defines GroundingRequest but does not explicitly state Send+Sync bounds — inferred from architecture.

2B.2 VERIFY: compilation + regression check
  PRE_GATE: 2B.1 complete; workspace compiles.
  TASK:
    - cargo check on full workspace
    - reasoning/ deps: only core/, tools/, thiserror, serde (no providers/ yet, no knowledge/)
      providers/ dep may come in 2C (per-agent config may need model info) or stay out
    - No circular dep: tools/ must NOT depend on reasoning/
  DETERMINISTIC_CHECK:
    - cargo check: 0 errors
    - No circular: grep "reasoning" tools/Cargo.toml = 0
    - reasoning/ deps: only core, tools, thiserror, serde (count = 0 extras)
  ADVERSARIAL:
    - reasoning/ accidentally pulls knowledge/ via tools/ transitive dep = compile time increase + coupling spread. cargo tree -p reasoning | grep knowledge must be empty.
  ACCEPTANCE:
    - cargo check exit 0
    - deps: core/, tools/, thiserror, serde
    - No circular (tools/ doesn't depend on reasoning/)

2B.3 TEST: write failing tests (TDD) — 16 tests
  PRE_GATE: 2B.2 passes; Verifier trait exists.
  TASK: Write tests in reasoning/tests/verification_tests.rs (all must FAIL against stubs):

    T1. file_check_existing_file: verify(FileCheck, "path/to/temp", "") -> passes=true, evidence contains path. Invariant reasoning.txt L46.

    T2. file_check_nonexistent_file: verify(FileCheck, "nonexistent_file_xyz.rs", "") -> passes=false.

    T3. source_check_function_exists: temp file with "fn hello_world()". verify(SourceCheck, "hello_world", path) -> passes=true, evidence contains "hello_world".

    T4. source_check_function_missing: temp file without matching fn. verify(SourceCheck, "nonexistent_fn", path) -> passes=false.

    T5. test_check_passing_test: verify(TestCheck, "test_name", "test_module_path") -> passes reflects actual result. Mock test runner or pre-existing passing test.

    T6. web_check_stub_returns_error: verify(WebCheck, "some algorithm", "") in unit test (no network) -> Err(VerificationError) or stub indicating network unavailable. Invariant R-2B-2.

    T7. api_check_stub_returns_error: same as T6 for ApiCheck.

    T8. api_registry_check_stub_returns_error: same for ApiRegistryCheck.

    T9. plan_grounding_sweep_blocks_ungrounded: plan with ["file:src/main.rs" (exists), "file:nonexistent.rs" (missing)]. ground_plan(plan). all_grounded == false. ungrounded contains "file:nonexistent.rs". Invariant reasoning.txt L199.

    T10. plan_grounding_sweep_passes_all_grounded: plan with only valid file claims. all_grounded == true, ungrounded empty.

    T11. plan_grounding_sweep_empty_plan: ground_plan(""). all_grounded == true. Invariant R-2B-3.

    T12. uncertain_claim_auto_resolution: ThinkStep grounding "training:some_claim" (UNCERTAIN). Claim references existing file. After verification, level promoted to Verified.

    T13. verification_method_requires_network: WebCheck.requires_network() == true. FileCheck.requires_network() == false.

    T14. verifier_trait_object_safety: let _: Box<dyn Verifier> = Box::new(LocalVerifier::new(...)). Must compile. LocalVerifier::new takes Box<dyn TestRunner> — pass MockTestRunner.

    T15. plan_sweep_adversarial_regex: plan with embedded "file:fake_path" inside comment/string literal plus real "file:src/main.rs". Example: "// see file:fake_path for details\nGrounding: file:src/main.rs". ground_plan extracts only real claim.

    T16. verify_claim_strips_prefix: verify_claim("file:src/main.rs", "", "") via GrcEngine. Verifier receives "src/main.rs" (no "file:" prefix). Validates BLOCK fix: prefix stripping in I3.

  DETERMINISTIC_CHECK:
    - >= 16 tests exist
    - Tests compile (types valid)
    - Tests fail at RUNTIME (stubs panic via todo!())
    - No network calls in tests (grep reqwest\|hyper\|TcpStream\|connect\|http:// = 0)
    - No timing-dependent tests (grep sleep\|Duration\|Instant\|SystemTime = 0)

  ADVERSARIAL:
    - T9 passes against stubs = stub too permissive (returning empty vec). Use todo!()/unimplemented!().
    - T12 auto-resolution uses real FS without temp dir cleanup = leak. tempfile crate or std::env::temp_dir cleaned in drop.
    - T5 runs actual cargo test in subprocess = runtime explodes. T5 must use mock test runner.

  ACCEPTANCE:
    - >= 16 test functions
    - all fail against stubs (TDD)
    - zero network calls
    - zero timing-dependent
    - all temp files via tempfile crate

2B.4 IMPLEMENT: verification methods + plan sweep
  PRE_GATE: 2B.3 complete; mcp_session_claim at EXECUTING.
  TASK:

    I1. LocalVerifier implementation (FILE_CHECK, SOURCE_CHECK, TEST_CHECK):
        LocalVerifier takes Box<dyn TestRunner> at construction (DI for D8).

        FILE_CHECK:
          use tools::read_file(path, None, None) to check existence.
          Claim format: file path ("file:" prefix stripped before passing to read_file).
          FileReadResult returned: passes=true, evidence="File exists: {path}, {total_lines} lines".
          tools returns Error::NotFound: passes=false, evidence="File not found: {path}".
          Grounding: tools.txt s2 L23 defines read_file(path, start_line?, end_line?) -> FileReadResult.

        SOURCE_CHECK:
          use tools::read_file(path, None, None) + string search.
          Claim = symbol_name (from grounding string). Context = file_path (from verify_claim context, D7).
          Read file, search for symbol patterns:
            regex: r"(fn|struct|enum|trait|impl|const|static|type|mod)\s+{symbol_name}\b"
          Found: passes=true, evidence="Found {kind} {name} at {file}:{line}".
          Not: passes=false, evidence="Symbol {name} not found in {file}".

        TEST_CHECK:
          delegate to injected TestRunner (D8).
          Claim = test_name. Context = crate_name.
          Production: CargoTestRunner calls BashSandbox::execute("cargo test --exact {name} -p {crate}", sandbox_config) with 30s timeout.
          Grounding: tools.txt s9 L184-186.
          BashResult.exit_code == 0: passes=true, evidence="Test {name} passed".
          Non-zero: passes=false, evidence="Test {name} failed: {stderr}".
          Timeout: passes=false, evidence="Test {name} timed out after 30s".

    I2. NetworkVerifier stubs (WEB_CHECK, API_CHECK, API_REGISTRY_CHECK):
        Phase 2B implements trait + dispatch, NOT real network calls.
        WEB_CHECK: returns VerificationError("network verification not implemented in phase 2B").
        API_CHECK: same stub.
        API_REGISTRY_CHECK: same stub.
        Rationale: real network needs provider integration, deferred to when providers/ connected to reasoning/ or 4A provides IPC proxy.
        Verifier trait + dispatch complete; only network backends stubbed.

    I3. verify_claim implementation on BasicGrcEngine:
        Replace unimplemented!() from 2A.
        Signature (D7): fn verify_claim(&self, claim: &str, source: &str, context: &str) -> Result<VerificationResult, GrcError>
        Parse claim to determine VerificationMethod (from "type:reference" format):
          "file:..." -> FileCheck (strip "file:" prefix)
          "grep:..." -> FileCheck (grep is file-based, strip "grep:" prefix)
          "test:..." -> TestCheck (strip "test:" prefix, use context as crate name)
          "search:..." -> WebCheck (strip "search:" prefix)
          "tool:..." -> SourceCheck (strip "tool:" prefix, use context as file path)
          "training:..." -> no verification (flag UNCERTAIN, return VerificationResult with passes=false)
        Dispatch via self.verifier.verify(method, stripped_claim, context).
        BasicGrcEngine holds Box<dyn Verifier> (injected at construction for testability).
        NOTE: "type:" prefix ALWAYS stripped before passing to Verifier. Verifier receives raw reference, not grounding format string.

    I4. ground_plan implementation on BasicGrcEngine:
        Replace unimplemented!() from 2A.
        Signature (D6): fn ground_plan(&self, plan: &str) -> PlanGroundingResult
        Algorithm:
          1. Parse plan text into claims using regex (pattern: "type:reference" where type in {file,grep,test,search,tool,training}).
          2. For each claim, call verify_claim().
          3. Classify: passes=true -> GROUNDED. passes=false -> UNGROUNDED. Type "training" -> VERIFY_NEEDED (attempt FILE_CHECK/SOURCE_CHECK auto-resolution; if fails, UNGROUNDED).
          4. Construct PlanGroundingResult { claims, all_grounded, ungrounded, verify_needed }.
          5. Empty plan (0 claims) -> PlanGroundingResult { all_grounded: true, ... }.
        verified_at: from injected TimeSource (core::entropy).

    I5. UNCERTAIN claim auto-resolution:
        When GRC encounters step with GroundingLevel::Uncertain:
          Extract claim reference from grounding string.
          Attempt FILE_CHECK (if path-like) or SOURCE_CHECK (if symbol-like).
          If verification passes: promote to Verified.
          If fails or not file/source-resolvable: keep Uncertain.
        Called during think() for Uncertain grounding steps (enhances 2A logic).

  DETERMINISTIC_CHECK:
    - All 16 tests pass (0 failed)
    - Phase 2A tests still pass (regression)
    - No todo!()/unimplemented!() in verification.rs or engine.rs verify_claim/ground_plan
    - No unwrap() in verification.rs
    - Network stubs return VerificationError (not panic)

  ADVERSARIAL:
    - verify_claim passes "file:src/main.rs" (including prefix) = looks for literal "file:src/main.rs". T1/T2 use actual paths without "file:" prefix in FILE_CHECK call.
    - ground_plan regex too greedy: matches "file:" inside comments. T15 covers.
    - TEST_CHECK spawns cargo with no timeout = hanging test blocks indefinitely. T5 mock verifies timeout set.
    - Box<dyn Verifier> not injected = cannot test with mocks. Constructor takes verifier param.

  ACCEPTANCE:
    - all 16+ tests pass
    - all 2A tests still pass
    - no todo!/unimplemented!/unwrap()/panic! in verification.rs
    - verify_claim and ground_plan no longer stubs
    - network stubs return typed errors

2B.5 MEASURE: verification performance
  PRE_GATE: 2B.4 complete.
  TASK:
    - FILE_CHECK latency (target <2ms for local existence)
    - SOURCE_CHECK latency (target <10ms for 1000-line file)
    - ground_plan latency for 20 claims (target <100ms)
    - Record: test count, LOC, compile time delta from 2A
    - Record: methods_implemented vs stubbed (3/6)
    - Write .elai_cc/metrics/phase_2B.toml

  DETERMINISTIC_CHECK:
    - phase_2B.toml exists with file_check_latency_us, source_check_latency_us, plan_sweep_latency_us, methods_implemented
    - FILE_CHECK < 2000us
    - SOURCE_CHECK < 10000us
    - plan_sweep < 100000us

  ADVERSARIAL:
    - SOURCE_CHECK benchmark uses 10-line file = unrealistic. Must use >= 1000-line file.
    - plan_sweep benchmark with 1 claim doesn't test scaling. Must use 20 claims.

  ACCEPTANCE:
    - phase_2B.toml exists
    - FILE_CHECK < 2000us
    - SOURCE_CHECK < 10000us
    - plan_sweep < 100000us
    - cargo test: 0 failures

PHASE 2B EXIT CRITERIA
  All 6 verification methods have type-level representation (VerificationMethod enum).
  3 methods fully implemented (FILE_CHECK, SOURCE_CHECK, TEST_CHECK).
  3 methods stubbed with typed errors (WEB_CHECK, API_CHECK, API_REGISTRY_CHECK).
  verify_claim dispatches to correct verifier based on grounding format.
  ground_plan decomposes plan into claims, verifies each, returns PlanGroundingResult.
  Ungrounded claims detected and reported (all_grounded == false when any fails).
  UNCERTAIN grounding auto-resolved via FILE_CHECK/SOURCE_CHECK where possible.
  Empty plan handled (trivially grounded, no panic).
  All 16+ tests pass. All 2A tests still pass.
  Zero todo!/unimplemented!/unwrap()/panic! in verification code.
  No network calls in any test.

PHASE 2B GROUNDING
  "6 verification methods" -> GROUNDED reasoning.txt s4 L46-55
  "Binary pass/fail verification" -> GROUNDED MiniCheck (EMNLP 2024, arxiv:2404.10774). Google Vertex AI grounding binary. Our methods deterministic (file exists or not), inherently binary.
  "Structural claim decomposition (regex, not NLI)" -> GROUNDED RARR (arxiv:2210.08726) uses LLM-based decomposition for semantic claims. Our claims are typed structural references ("type:reference" from 2A). Regex extraction deterministic.
  "Plan grounding sweep blocks EXECUTING" -> GROUNDED reasoning.txt s4 L64-65, s11 L199
  "GroundingResult includes verified_at" -> GROUNDED reasoning.txt L184
  "tools/ as reasoning/ dependency" -> GROUNDED implementation-guide.txt L64 crate map
  "No circular dep (tools -> knowledge, knowledge !-> reasoning)" -> VERIFIED implementation-guide.txt L65 (tools depends on core, knowledge), L64 (reasoning depends on core, providers, tools). knowledge does not depend on reasoning.
  "Verifier trait must be Send + Sync" -> VERIFIED (inference): 4A uses process isolation with IPC (reasoning.txt s9 L134). IPC implies cross-thread. L137 defines GroundingRequest but does not state Send+Sync — inferred.
  "FILE_CHECK <2ms, SOURCE_CHECK <10ms targets" -> UNGROUNDED (verify empirically in 2B.5)
  "3/6 methods implemented in 2B, 3 network-dependent stubbed" -> GROUNDED reasoning.txt s9 L134 (agents have no network access except researcher). Network methods need orchestrator proxy (4A).


==============================================================
PHASE 2C: PER-AGENT GRC CONFIGURATION
==============================================================

Prerequisites: Phase 2B complete (grounding verification methods implemented, plan sweep blocks ungrounded claims).

GOAL
Different agent roles get different GRC budgets, step sequences, grounding constraints,
allowed verification methods. GRC engine routes config by role at spawn time. Auditor gets
double-verify with VERIFIED-only grounding. Context-compiler gets observe-only with
FILE_CHECK/SOURCE_CHECK only. Config layered: global defaults in elai_cc.toml, per-role
overrides in [agents.{role}.grc], validated at load time.

READ BEFORE IMPLEMENTING
  docs/specs/crates/reasoning.txt Section 7 (per-agent config: 5 role definitions)
  docs/specs/crates/reasoning.txt Section 9 (GRC under process isolation: IPC proxy)
  docs/specs/crates/reasoning.txt Section 10 (KEY TYPES: AgentGrcConfig)
  docs/specs/crates/reasoning.txt Section 11 (TESTABLE INVARIANTS item 9 is 2C)
  docs/specs/architecture-overview.txt Section 9 (agent roles and model assignments)
  docs/specs/crates/orchestrator.txt Section 7 (agent supervision, checkpoint, restart)
  docs/specs/crates/orchestrator.txt Lines 440-444 (budget enforcement in orchestrator)
  docs/specs/integration/prompt-architecture.txt Lines 61-86 ([agents.{role}] TOML pattern)

DESIGN DECISIONS (grounded)
  D1. Per-agent transition maps extending GrcState::allowed_next().
      Auditor requires [observe, hypothesize, verify, verify] — double verify.
      Phase 2A's allowed_next() does NOT permit verify->verify. Without extension, auditor's required_steps trigger InvalidTransition on second verify.
      Solution: AgentGrcConfig carries optional transition_overrides: HashMap<GrcState, Vec<GrcState>> that engine merges with default allowed_next() for that agent's session.
      Auditor override: {Verify: [Verify, Observe]}.
      Additive (cannot remove default, only add) to prevent misconfiguration from breaking valid chains.
      Grounding: reasoning.txt s7 L97 specifies auditor required_steps includes double-verify. Phase 2A has no Verify->Verify path. Without this, spec unimplementable.

  D2. Config validation at load time (fail-fast).
      AgentGrcConfig validated when parsed from elai_cc.toml, not at first GRC call.
      Rules:
        (a) required_steps non-empty
        (b) required_steps forms valid chain through default + overridden transitions
        (c) budget.max_calls >= required_steps.len()
        (d) if csq_required=true, required_steps must include at least one Verify
        (e) allowed_methods subset of all known VerificationMethod variants
      Provable benefit: catches impossible configs (budget=1 with 5 required_steps) before work starts.

  D3. TOML layering: global defaults + per-role overrides.
      Global GRC defaults in [grc.defaults]. Per-role overrides in [agents.{role}.grc].
      Merge: per-role values override globals field-by-field (not deep merge). Missing per-role fields inherit from global.
      Matches existing pattern: [agents.{role}.supervision] (orchestrator), [agents.{role}.slo_tier] (providers), [agents.{role}.sandbox] (tools), [agents.{role}.role_prompt] (prompt-architecture).
      NOTE: existing TOML keys use underscores (e.g., [agents.context_compiler], [agents.prompt_engineer]) per prompt-architecture.txt L74-82. Phase 2C MUST follow: [agents.context_compiler.grc], NOT [agents.context-compiler.grc]. Hyphens would be valid TOML but silently mismatched.

  D4. Allowed verification methods per role (POLICY layer).
      AgentGrcConfig.allowed_methods: Vec<VerificationMethod>. POLICY restriction: limits which grounding methods agent permitted to request, independent of infrastructure.
      Distinction from s9 (CAPABILITY layer): s9 L140-146 describes IPC proxy capabilities — what orchestrator CAN provide to each role (e.g., "WEB_CHECK: Available to all" via IPC). allowed_methods is what role's GRC config PERMITS it to request. Policy <= Capability.
      Context-compiler policy: [FILE_CHECK, SOURCE_CHECK] only (reasoning.txt s7 L105-106), even though orchestrator could proxy WEB_CHECK.
      Executor/auditor policy: all methods (matching full capability set).
      Phase 2C: policy enforced in-process (GRC engine rejects disallowed method requests before they reach orchestrator). Phase 4A adds second enforcement layer on GroundingRequest IPC.
      SPEC UPDATE NEEDED: reasoning.txt L186 AgentGrcConfig should add allowed_methods and transition_overrides. (Cannot edit reasoning.txt — owned by session s-1003192.)

  D5. Anti-rubber-stamp enforcement for auditor.
      Auditor "must find >= 1 issue per review" (reasoning.txt L99). Not just prompt — enforced in GRC.
      When auditor's GRC session reaches Converge, engine checks at least one ThinkStep has step_type=Verify with a finding (content contains structured marker, e.g., "FINDING:" prefix). If no finding: Converge rejected with new GrcError variant InsufficientFindings.
      Provable benefit: without enforcement, anti-rubber-stamp is advisory-only; auditor model can skip verification with no consequence. Auditor is last line of defense before code accepted — rubber-stamp auditor defeats entire adversarial review architecture.


STEPS

2C.1 SKELETON: AgentGrcConfig type + config loading stubs
  PRE_GATE:
    - Phase 2B tests pass
    - GrcEngine trait exists with think/set_budget/get_budget/reset_topic
    - VerificationMethod enum exists (from 2B)
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
        require_scope_enumeration: when true, Observe step ThinkStep.scope must be populated with all task components/modalities. Orchestrator validates every scope item appears in subsequent plan_action. Unaddressed items -> WARN.

    - Add GrcError::InsufficientFindings { role: String, expected_min: usize }
    - Add GrcError::ConfigValidation { role: String, reason: String }

    - Define AgentGrcConfigLoader trait in reasoning/src/config.rs:
        fn load_config(role: &str, global_defaults: &GrcDefaults) -> Result<AgentGrcConfig, GrcError>
        fn validate_config(config: &AgentGrcConfig) -> Result<(), GrcError>

    - Define GrcDefaults struct:
        { budget: GrcBudget, grounding_level: GroundingLevel, csq_required: bool, allowed_methods: Vec<VerificationMethod> }

    - Stub implementation: TomlConfigLoader reads from elai_cc.toml

    - Add per-role config routing to GrcEngine:
        fn with_agent_config(&mut self, config: AgentGrcConfig)
        fn get_agent_config(&self) -> Option<&AgentGrcConfig>

    - Re-export from reasoning/src/lib.rs

  DETERMINISTIC_CHECK:
    - cargo check: 0 errors
    - AgentGrcConfig struct exists (1)
    - InsufficientFindings variant
    - ConfigValidation variant
    - load_config/validate_config in config.rs
    - GrcEngine.with_agent_config method

  ADVERSARIAL:
    - AgentGrcConfig uses String for role instead of enum = typos undetected (e.g., "exector"). Consider AgentRole enum but verify no conflict with orchestrator role definitions. Decision: use String for now (cross-crate enum coupling worse), validate against known roles at load time.
    - transition_overrides allows removing default transitions (not just adding) = misconfigured role could break base GRC chain. Validate additive-only (union, not replacement).

  ACCEPTANCE:
    - cargo check: 0 errors
    - AgentGrcConfig has 8 fields (role, budget, required_steps, grounding_level, csq_required, allowed_methods, transition_overrides, require_scope_enumeration)
    - GrcError has InsufficientFindings + ConfigValidation

2C.2 VERIFY: compilation check
  PRE_GATE: 2C.1 complete.
  TASK:
    - cargo check on full workspace
    - reasoning/ does not gain new crate deps (still core/ + 2B deps)
    - No new external deps (toml parsing comes from core/ Config)
  DETERMINISTIC_CHECK:
    - cargo check: 0 errors
    - No new dep crates (diff Cargo.toml: 0 or minimal additions, no toml)
  ADVERSARIAL:
    - reasoning/ depends on toml crate directly = bypasses core::Config, parallel config path. reasoning/ should receive parsed AgentGrcConfig from orchestrator, not parse TOML itself. TOML parsing in core/ or orchestrator/ — reasoning/ receives typed structs.
  ACCEPTANCE:
    - cargo check exit 0
    - reasoning/ Cargo.toml has no toml/toml_edit dep

2C.3 TEST: write failing tests (TDD) — 15 tests
  PRE_GATE: 2C.2 passes; AgentGrcConfig defined.
  TASK: Write tests in reasoning/tests/agent_config.rs (all must FAIL against stubs):

    T1. auditor_gets_double_verify:
        AgentGrcConfig auditor, required_steps=[Observe,Hypothesize,Verify,Verify], transition_overrides={Verify: [Verify, Observe]}.
        with_agent_config. think(Observe), think(Hypothesize), think(Verify), think(Verify).
        All 4 succeed. Second Verify does NOT return InvalidTransition.
        Invariant reasoning.txt s7 L97, s11 item 9.

    T2. context_compiler_observe_only:
        AgentGrcConfig context-compiler, required_steps=[Observe].
        think(Observe) succeeds. think(Hypothesize) -> Err(InvalidTransition).
        Invariant reasoning.txt s7 L103-106.

    T3. executor_budget_8_1200:
        AgentGrcConfig executor, budget=(8, 1200). get_budget returns max_calls=8, max_chars=1200. think() 8 times. 9th -> BudgetExhausted.

    T4. memory_writer_budget_2_400:
        AgentGrcConfig memory-writer, budget=(2, 400). think() twice. 3rd -> BudgetExhausted.

    T5. config_validation_empty_steps_rejected:
        AgentGrcConfig with required_steps=[]. validate_config() -> Err(ConfigValidation) with reason mentioning empty steps.

    T6. config_validation_budget_too_small:
        AgentGrcConfig with budget.max_calls=2 and required_steps with 5 entries. validate_config() -> Err(ConfigValidation) with reason mentioning budget insufficient.

    T7. auditor_anti_rubber_stamp:
        auditor config. Observe, Hypothesize, Verify (content NO "FINDING:" marker), attempt Converge. Converge -> Err(InsufficientFindings). Add Verify step with "FINDING: unused variable" in content, then Converge. Converge succeeds.
        Invariant reasoning.txt s7 L99.

    T8. auditor_rejects_uncertain_grounding:
        auditor config grounding_level=Verified. think(Observe, grounding="training:unverified_claim") -> error (Uncertain below minimum Verified).

    T9. context_compiler_allowed_methods:
        context-compiler allowed_methods=[FILE_CHECK, SOURCE_CHECK]. Request WEB_CHECK grounding -> rejected.

    T10. global_defaults_with_per_role_override:
        GrcDefaults budget=(5, 800). Load executor config overriding budget to (8, 1200) but inheriting other defaults. executor budget (8, 1200); other fields match defaults.

    T11. transition_overrides_are_additive:
        transition_overrides={Observe: [Verify]}. think(Observe). Next allowed states include BOTH Hypothesize (default) AND Verify (override). Not just Verify.

    T12. auditor_budget_10_1200:
        auditor budget=(10, 1200). think() 10 times. 11th -> BudgetExhausted.
        Invariant reasoning.txt s7 L96.

    T13. prompt_engineer_budget_5_800:
        prompt-engineer budget=(5, 800), required_steps=[Observe, Hypothesize, Converge]. think() 5 times. 6th -> BudgetExhausted.
        Invariant reasoning.txt s7 L109-111.

    T14. designer_config_distinct_from_executor:
        Designer budget=(8, 1200), required_steps=[Observe, Hypothesize, PlanAction, Verify, Converge], require_scope_enumeration=true.
        Designer config differs from executor in role name, required_steps, role_prompt (Designer produces PipelineSpec, not code).
        Designer reuses executor model but has separate AgentGrcConfig.
        Invariant technical-analysis-v3.txt Concern 5, reasoning.txt s7.

    T15. require_scope_enumeration_per_role:
        executor require_scope_enumeration=true; memory_writer require_scope_enumeration=false.
        executor Observe requires scope field; memory_writer's does not.

  DETERMINISTIC_CHECK:
    - >= 15 tests exist and FAIL (TDD)
    - Test count: grep -c "#\[test\]" agent_config.rs >= 15

  ADVERSARIAL:
    - Tests pass against stubs = too permissive. todo!()/unimplemented!().
    - T7 exact string matching on "FINDING:" brittle. Use structured FindingMarker type or case-insensitive prefix check. Document contract.
    - T11 does not verify default transitions preserved = additive-only could regress silently if merge replaces.

  ACCEPTANCE:
    - >= 15 tests in reasoning/tests/agent_config.rs
    - all fail to compile or at runtime
    - zero timing-dependent tests

2C.4 IMPLEMENT: config loading, per-role routing, transition extension
  PRE_GATE: 2C.3 complete; mcp_session_claim.
  TASK:

    I1. Config validation (config.rs):
        validate_config() checks all 5 rules from D2:
          (a) required_steps non-empty
          (b) required_steps forms valid chain through merged transitions
          (c) budget.max_calls >= required_steps.len()
          (d) csq_required implies at least one Verify in required_steps
          (e) allowed_methods subset of VerificationMethod variants
        Returns Err(ConfigValidation) on first failure with specific reason.

    I2. Transition override merging + step whitelist (engine.rs):
        When AgentGrcConfig set via with_agent_config, engine stores config.
        On each think(), effective_allowed_next(state):
          base: GrcState::allowed_next(state)
          plus: config.transition_overrides.get(state).unwrap_or_default()
          merged: base.union(overrides) — additive only.
        THEN: if config.required_steps non-empty, filter merged set to only include states in required_steps (plus Status and Converge, always allowed). Step whitelist: restricts agent to only step types its role permits. Without filter, context-compiler's required_steps=[Observe] has no effect — default allowed_next(Observe)=[Hypothesize] still allows Hypothesize. Filter ensures only Observe (plus Status/Converge) reachable.
        Whitelist: reachable_states = required_steps + [Status, Converge].
        effective_allowed = merged.intersect(reachable_states).

    I3. Grounding level enforcement (engine.rs):
        On think() with grounding, classify GroundingLevel.
        If level < config.grounding_level (e.g., Uncertain < Verified), reject step.
        New error variant or extend UngroundedStep with "level_insufficient" reason.
        Ordering: Verified > High > Uncertain. Implement PartialOrd on GroundingLevel.
        CRITICAL: enum variant order MUST be { Uncertain=0, High=1, Verified=2 } so derived PartialOrd gives Uncertain < High < Verified. If declared as { Verified, High, Uncertain } (2A default), derived Ord inverts: Verified=0 < Uncertain=2, and `level < config.grounding_level` passes when it should reject. Phase 2A's GroundingLevel enum order must be corrected or explicit PartialOrd impl provided (not derived).
        Test: assert!(GroundingLevel::Verified > GroundingLevel::Uncertain) — add to T8.

    I4. Anti-rubber-stamp (engine.rs):
        On think(Converge) when agent config role == "auditor":
          scan session history for Verify steps with "FINDING:" prefix in content.
          if count < 1: return Err(InsufficientFindings { role, expected_min: 1 }).
        FINDING_PREFIX: &str = "FINDING:" const.
        Contract: finding = any Verify step whose content starts with FINDING_PREFIX (case-sensitive, after trimming leading whitespace). Internal protocol between GRC engine and auditor prompt — not NL parsing.
        Tests must cover: exact prefix, leading whitespace, content without prefix.

    I5. Config loading from TOML (config.rs):
        TomlConfigLoader::load_config(role, defaults):
          reads from core::Config (already parsed from elai_cc.toml)
          merges [grc.defaults] with [agents.{role}.grc] per D3
          calls validate_config() before returning
        If role not found in config: use defaults only (no error — allows dynamic roles).
        If role found but validation fails: return error.

    I6. Per-role routing (engine.rs):
        with_agent_config() stores config and resets topic state.
        All subsequent think() calls use config's budget, transitions, grounding level.
        get_agent_config() returns current config (for orchestrator inspection).
        If no agent config set, engine uses global defaults (backward-compatible with 2A).
        BUDGET ENFORCEMENT SCOPE: in 2C (in-process, pre-isolation), agent-side GrcEngine enforces budgets directly. In 4A (Tier C process isolation), orchestrator.txt L440-444: "Agent binary budget tracking is advisory only. Enforcement is in the orchestrator." Phase 2C's enforcement is AUTHORITATIVE for phases 0-3 (in-process). Phase 4A transitions authority to orchestrator and demotes agent-side to advisory. This phase does NOT anticipate that demotion — 4A handles transition. Same GrcBudget struct in both modes; only who enforces changes.

  DETERMINISTIC_CHECK:
    - All 15 tests pass (0 failed)
    - All 2A + 2B tests still pass (no regression)
    - No todo!()/unimplemented!() in config.rs or engine.rs
    - No unwrap() in config.rs or engine.rs
    - GroundingLevel implements PartialOrd

  ADVERSARIAL:
    - FINDING_PREFIX hardcoded string matching = output format drift breaks detection. Use structured FindingReport type if feasible, or document contract + test varied formatting (leading whitespace, mixed case). Decision: keep simple prefix (internal protocol, not NL). Document in code comment.
    - with_agent_config() does not reset topic state = leftover state from previous config leaks. Test: set A, run steps, set B, verify topic state clean.
    - Config loading fails silently (returns defaults on parse error) = misconfigurations undetected. Malformed [agents.executor.grc] must return error, not silently fall back. Only missing table falls back.
    - PartialOrd wrong ordering (Uncertain > Verified) = auditor accepts untested claims. Assert Verified > High > Uncertain explicitly.

  ACCEPTANCE:
    - cargo test: 0 failures (all 2A+2B+2C)
    - cargo check: 0 errors
    - no todo!/unimplemented!/unwrap() in engine.rs/config.rs
    - GroundingLevel has PartialOrd
    - FINDING_PREFIX constant defined

2C.5 MEASURE: config overhead and correctness metrics
  PRE_GATE: 2C.4 complete.
  TASK:
    - Config load + validation time per role (target <1ms — pure struct + validation)
    - Per-step overhead from config routing vs no-config baseline (target <10%)
    - Memory per AgentGrcConfig (target <2KB including transition_overrides)
    - Record test count, LOC delta from 2B, compile time delta
    - Write .elai_cc/metrics/phase_2C.toml

  DETERMINISTIC_CHECK:
    - phase_2C.toml exists with config_load_us
    - config_load_us < 1000
    - routing_overhead_pct < 10.0

  ADVERSARIAL:
    - Measurement includes TOML parsing = conflates I/O with config logic. Measure config validation separately. Report both.
    - AgentGrcConfig with large transition_overrides may exceed 2KB. Measure with max reasonable overrides (7 states x 7 targets = 49 entries).

  ACCEPTANCE:
    - phase_2C.toml exists
    - config_load_us < 1000
    - routing_overhead_pct < 10.0
    - cargo test: 0 failures

PHASE 2C EXIT CRITERIA
  Each of 6 agent roles (including Designer) has correct GRC budget and step sequence.
  Designer role distinct AgentGrcConfig from Executor (same model, different role_prompt/steps).
  Auditor gets double-verify via transition_overrides (verify->verify allowed).
  Context-compiler gets observe-only (hypothesize/plan_action/act/verify rejected).
  Auditor anti-rubber-stamp enforced: converge without finding rejected.
  Grounding level enforcement: auditor rejects Uncertain, context-compiler accepts High.
  Config validation catches: empty steps, budget < steps, missing verify for csq, invalid methods.
  Global defaults + per-role overrides merge field-by-field (not deep merge).
  Transition overrides additive-only (default transitions preserved).
  require_scope_enumeration enforced per-role (true for executor/designer, false for memory_writer).
  All 2A + 2B tests still pass.
  All 15+ 2C tests pass. Zero todo!/unimplemented!/unwrap() in new code.

PHASE 2C GROUNDING
  "Auditor required_steps includes double-verify" -> GROUNDED reasoning.txt s7 L97
  "Context-compiler is observe-only with FILE_CHECK/SOURCE_CHECK" -> GROUNDED reasoning.txt s7 L103-106
  "Executor budget 8/1200, memory-writer 2/400" -> GROUNDED reasoning.txt s7 L89-91, 115-116
  "Global default configurable in elai_cc.toml" -> GROUNDED reasoning.txt s3 L38
  "[agents.{role}] TOML pattern" -> VERIFIED grep 5 specs
  "Anti-rubber-stamp: must find >= 1 issue" -> GROUNDED reasoning.txt s7 L99
  "Per-agent FSM sub-states: IDLE->LOADING->REASONING->ACTING->DONE" -> GROUNDED reasoning.txt s7 L118-120
  "AgentGrcConfig type has 4 fields" -> GROUNDED reasoning.txt s10 L186. GAP: missing allowed_methods, transition_overrides, require_scope_enumeration — spec update needed. Now 8 fields total.
  "require_scope_enumeration field on AgentGrcConfig" -> GROUNDED reasoning.txt s10, technical-analysis-v3.txt Concern 3 Mechanism 1
  "Designer is distinct AgentRole from Executor" -> GROUNDED technical-analysis-v3.txt Concern 5, reasoning.txt s7
  "Orchestrator enforces budget externally via IPC message counting" -> GROUNDED orchestrator.txt L440-444
  "Per-agent config routing prevents InvalidTransition" -> VERIFIED without transition_overrides, auditor's verify->verify hits 2A InvalidTransition
  "Config validation at load time (fail-fast)" -> GROUNDED standard engineering. Provable: budget=1 with 5 required_steps fails at runtime without validation.
  "Anti-rubber-stamp defeats adversarial review if advisory-only" -> GROUNDED reasoning.txt s7 L99. Without enforcement, prompt compliance optional.
  "Role-based agent config is standard" -> VERIFIED MetaGPT, CrewAI (2025 web search)
  "<1ms config load overhead" -> UNGROUNDED (verify in 2C.5)
  "<10% per-step routing overhead" -> UNGROUNDED (verify in 2C.5)

PHASE 2C SPEC GAPS IDENTIFIED
  1. reasoning.txt L186 AgentGrcConfig should add allowed_methods: Vec<VerificationMethod>, transition_overrides: HashMap<GrcState, Vec<GrcState>>, and require_scope_enumeration: bool. Currently has 4 fields, needs 8. (reasoning.txt owned by session s-1003192 — coordinate update.)
  2. reasoning.txt s11: testable invariant for anti-rubber-stamp enforcement not listed. Should add "[2C] Auditor converge without finding is rejected (InsufficientFindings)."
  3. reasoning.txt GrcError enum (L174): missing InsufficientFindings and ConfigValidation variants.
  4. reasoning.txt s10 (GroundingLevel): variant order must be { Uncertain, High, Verified } (ascending quality = ascending discriminant) for derived PartialOrd. Current order { Verified, High, Uncertain } inverts comparison. Phase 2A must fix before 2C can use PartialOrd for grounding level enforcement.
  5. reasoning.txt s7: grounding_level field semantic ambiguity. For context-compiler, restriction is on method type (FILE_CHECK/SOURCE_CHECK only), not grounding quality. grounding_level=High would accept WEB_CHECK results returning Verified quality. allowed_methods and grounding_level are complementary: allowed_methods restricts WHICH methods, grounding_level restricts MINIMUM QUALITY.


==============================================================
PHASE 2D: CSQ AND INLINE FEASIBILITY
==============================================================

Prerequisites: Phase 2C complete (per-agent GRC configs route correctly, all 2C tests pass).

GOAL
Counterfactual Safety Questions (CSQ) on functions touching files/data/network/auth.
Inline feasibility checks catch ungrounded claims during EXECUTING (max 2 GRC calls).
After 2D, reasoning engine is complete (phases 2A-2D).

READ BEFORE IMPLEMENTING
  docs/specs/crates/reasoning.txt Section 6 (CSQ: format, triggers, guard requirement)
  docs/specs/crates/reasoning.txt Section 8 (inline feasibility: detection, max 2 calls, BLOCK signal)
  docs/specs/crates/reasoning.txt Section 7 (per-agent CSQ config: which roles require CSQ)
  docs/specs/crates/reasoning.txt Section 10 (KEY TYPES items marked [2D])
  docs/specs/crates/reasoning.txt Section 11 (TESTABLE INVARIANTS items marked [2D])
  docs/specs/crates/orchestrator.txt Section 8 (FSM transitions: EXECUTING->PLANNING replan signals)
  docs/specs/architecture-overview.txt Section 4 (FSM flow: EXECUTING includes GRC inline feasibility)

DESIGN DECISIONS (grounded)
  D1. CSQ trigger detection via function signature analysis.
      CSQ must fire on functions touching files/network/money/auth (reasoning.txt s6).
      Detection: pattern match on function parameters and return types.
      Functions receiving Path/File/TcpStream/Money/AuthToken or calling std::fs/std::net APIs classified as CSQ-required.
      Phase 1A DKG provides call-graph data, but CSQ detection must work WITHOUT DKG (reasoning crate depends on core+providers+tools, not knowledge). Standalone pattern: accept list of "sensitive function signatures" from caller (orchestrator passes DKG results at runtime).
      Grounding: reasoning.txt s6 mandates "minimum 1 CSQ per function touching files/network/money/auth." Detection via call-graph analysis standard static analysis (ast-grep, tree-sitter).
      DEP note: reasoning/ depends on core/, providers/, tools/ (crate map). Does NOT depend on knowledge/. DKG queries injected by orchestrator at call time, not imported directly.

  D2. CSQ severity classification determines guard enforcement.
      Not all CSQ consequences equal. "Code writes wrong comment" vs "code deletes user data" differ.
      CRITICAL (guard MANDATORY): data_loss, security_breach, financial_error, auth_bypass
      WARNING (guard RECOMMENDED): incorrect_output, performance_degradation, stale_cache
      CRITICAL consequences no guard = CSQ failure (blocks AUDITING).
      WARNING consequences no guard = CSQ warning (logged, does not block).
      Grounding: reasoning.txt s6: "Dangerous consequence (data loss, security, financial) -> MUST add guard." Maps directly to CRITICAL. VeriGuard (OpenReview 2025) co-generates verification with code; CSQ is lightweight version applied at reasoning level.

  D3. Inline feasibility uses GRC budget counter, not separate mechanism.
      Inline feasibility (reasoning.txt s8) fires when GRC verify detects NEW claim not in PipelineSpec, Research Report, or DKG. Max-2-call limit enforced by dedicated counter in InlineFeasibilityState, separate from per-topic GRC budget. Rationale: inline checks are cross-topic (claim in topic A might need verification via topic B's context), so per-topic budget is wrong scope.
      Methods: web_search, grep, api_registry_check (reasoning.txt s8).
      Grounding: reasoning.txt s8 "Quick check (max 2 GRC calls)." AgentSpec (arxiv:2503.18666) validates runtime enforcement with bounded checks; our approach simpler (fixed counter vs probabilistic reachability).
      Receiver note: check_inline_feasibility takes &mut self (mutates counter). think, set_budget already use &mut self. verify_claim/ground_plan (2B) use &self (read-only). Callers already hold &mut GrcEngine for think(); no new API burden.

  D4. Inline feasibility failure signals replan via ReplanRequest.
      PASS: claim grounded, proceed with EXECUTING.
      FAIL: executor must use plan's original approach (no deviation).
      BLOCKED: (fundamentally different approach detected) executor emits ReplanRequest(MAJOR) which triggers EXECUTING->PLANNING transition requiring human approval (orchestrator.txt s8: "EXECUTING -> PLANNING (major replan): requires human approval in both AUTONOMOUS and INTERACTIVE modes").
      Grounding: reasoning.txt s8 "Fundamentally different approach requires EXECUTING -> BLOCK -> PLANNING."

  D5. CSQ and inline feasibility types extend reasoning/src/types.rs.
      New types (not yet in reasoning.txt s10, will be added):
        CsqCheck { assumption: String, consequence: String, severity: CsqSeverity, guard_added: bool }
        CsqSeverity enum { Critical, Warning }
        CsqResult { function_id: String, checks: Vec<CsqCheck> }
          fn passed(&self) -> bool — true iff all CRITICAL checks have guard_added == true.
          No stored passed field; computed method ensures consistency when checks mutated.
        FeasibilityMethod enum { SourceMatch }
          Typed enum following VerificationMethod pattern.
          Extensible: future phases add WebSearch, Grep, ApiRegistryCheck variants.
        InlineFeasibilityResult { claim: String, method: FeasibilityMethod, outcome: FeasibilityOutcome }
        FeasibilityOutcome enum { Pass, Fail, Blocked }
          Pass: claim verified. Fail: use original plan. Blocked: emit ReplanRequest(MAJOR).
        InlineFeasibilityState { checks_used: u32, max_checks: u32 }
          max_checks = 2 (hardcoded from spec). is_exhausted() = checks_used >= max_checks.
          reset() sets checks_used = 0 (orchestrator calls between phases).
          When exhausted, no more inline checks; treat new claims as Fail.
      Grounding: type fields derived directly from reasoning.txt s6/s8. CsqSeverity maps to s6 "dangerous consequence" distinction.


STEPS

2D.1 SKELETON: CSQ and inline feasibility types + stubs
  PRE_GATE:
    - Phase 2C tests pass
    - AgentGrcConfig has csq_required field (from 2C)
    - No CsqCheck type exists yet (greenfield)
  TASK:
    - Add types to reasoning/src/types.rs:

        CsqSeverity enum { Critical, Warning }
          #[derive(Debug, Clone, Copy, PartialEq, Eq)]

        CsqCheck {
          assumption: String,
          consequence: String,
          severity: CsqSeverity,
          guard_added: bool
        }
          #[derive(Debug, Clone)]

        CsqResult {
          function_id: String,
          checks: Vec<CsqCheck>
        }
          #[derive(Debug, Clone)]
          fn passed(&self) -> bool { self.checks.iter().all(|c| c.severity != Critical || c.guard_added) }
          No stored passed field — computed method ensures consistency.

        InlineFeasibilityState {
          checks_used: u32,
          max_checks: u32
        }
          #[derive(Debug, Clone, Copy, PartialEq, Eq)]
          Default: max_checks = 2
          fn is_exhausted(&self) -> bool { self.checks_used >= self.max_checks }
          fn record_check(&mut self) { self.checks_used = self.checks_used.saturating_add(1); }
          fn reset(&mut self) { self.checks_used = 0; }  // called by orchestrator between phases

        FeasibilityOutcome enum { Pass, Fail, Blocked }
          #[derive(Debug, Clone, Copy, PartialEq, Eq)]

        FeasibilityMethod enum { SourceMatch }
          #[derive(Debug, Clone, Copy, PartialEq, Eq)]
          Follows VerificationMethod pattern (reasoning.txt s10 L190, 194).
          Future: WebSearch, Grep, ApiRegistryCheck (4F).

        InlineFeasibilityResult {
          claim: String,
          method: FeasibilityMethod,
          outcome: FeasibilityOutcome
        }
          #[derive(Debug, Clone)]

    - Add methods to GrcEngine trait in reasoning/src/engine.rs:
        Phase 2D methods (implemented this phase):
          fn generate_csq(&self, function_id: &str, sensitive_ops: &[String]) -> Result<CsqResult, GrcError>
          fn check_inline_feasibility(&mut self, claim: &str, sources: &[String]) -> Result<InlineFeasibilityResult, GrcError>
        Stubs in BasicGrcEngine using todo!() until 2D.4.

    - Add InlineFeasibilityState field to BasicGrcEngine struct.

    - Re-export new types from reasoning/src/lib.rs

  DETERMINISTIC_CHECK:
    - cargo check: 0 errors
    - All new types exist:
      CsqSeverity, CsqCheck, CsqResult, InlineFeasibilityState, FeasibilityOutcome, FeasibilityMethod, InlineFeasibilityResult
    - GrcEngine has generate_csq, check_inline_feasibility

  ADVERSARIAL:
    - CsqCheck.guard_added defaults true = all checks auto-pass without actual guards. Require explicit guard_added=true from caller. No Default impl sets guard_added=true.
    - InlineFeasibilityState.max_checks configurable = callers could set u32::MAX. Hardcoded to 2 in Default impl, not settable via public constructor.

  ACCEPTANCE:
    - cargo check: exit 0
    - 7 new types in types.rs
    - 2 new methods in GrcEngine trait

2D.2 VERIFY: compilation check
  PRE_GATE: 2D.1 complete.
  TASK:
    - cargo check on full workspace
    - reasoning/ has no NEW deps beyond declared (core, providers, tools, thiserror, serde)
  DETERMINISTIC_CHECK:
    - cargo check: 0 errors
    - No new deps (count excluding declared = 0)
  ADVERSARIAL:
    - Stubs return Ok(default_value) instead of todo!() = tests pass falsely in 2D.3. grep todo!() in generate_csq and check_inline_feasibility.
  ACCEPTANCE:
    - exit 0
    - reasoning/ deps: only core, providers, tools, thiserror, serde (no new deps from 2D)

2D.3 TEST: write failing tests (TDD) — 10 tests
  PRE_GATE: 2D.2 passes; generate_csq + check_inline_feasibility exist.
  TASK: Write tests in reasoning/tests/csq_tests.rs (all must FAIL against stubs):

    T1. csq_generates_check_for_file_function:
        generate_csq("write_config", &["std::fs::write"]).
        result.checks.len() >= 1.
        At least one check has severity == Critical.
        Invariant reasoning.txt s6 "minimum 1 CSQ per function touching files."

    T2. csq_critical_without_guard_fails:
        CsqResult with one Critical check, guard_added == false. passed() == false.
        Invariant reasoning.txt s6 "dangerous consequence -> MUST add guard."

    T3. csq_critical_with_guard_passes:
        CsqResult with one Critical check, guard_added == true. passed() == true.

    T4. csq_warning_without_guard_still_passes:
        CsqResult with one Warning check, guard_added == false. passed() == true (warnings don't block).

    T5. csq_no_sensitive_ops_returns_empty:
        generate_csq("helper_fn", &[]). checks.is_empty(). passed() == true.

    T6. inline_feasibility_pass_on_known_claim:
        check_inline_feasibility("use tantivy BM25 index for full-text search", &["PipelineSpec: use tantivy BM25 index for full-text search"]).
        outcome == Pass.
        Note: realistic multi-word claim avoids trivial substring false positive.

    T7. inline_feasibility_fail_on_unknown_claim:
        check_inline_feasibility("unknown claim", &[]) with no matching sources.
        outcome == Fail.

    T8. inline_feasibility_max_two_calls:
        inline_feasibility_state.checks_used == 0 (initial).
        Call check_inline_feasibility() twice. checks_used == 2.
        Third call -> Err(BudgetExhausted).
        Invariant reasoning.txt s8 "max 2 GRC calls." reasoning.txt s11 "[2D] Inline feasibility max 2 calls (bounded)."

    T9. inline_feasibility_blocked_signals_replan:
        check_inline_feasibility("CONTRADICTS_PLAN: use completely different algorithm", &["use original algorithm"]).
        CONTRADICTS_PLAN: prefix is 2D trigger for Blocked detection (I2).
        outcome == Blocked.
        Invariant reasoning.txt s8 "Fundamentally different approach requires EXECUTING -> BLOCK -> PLANNING."
        Note: Blocked outcome signals need for ReplanRequest(MAJOR), but actual emission is orchestrator's responsibility (3A+). This test verifies reasoning engine returns correct outcome only.

    T10. inline_feasibility_state_reset:
         After exhausting (2 used), call reset on InlineFeasibilityState. checks_used == 0 and is_exhausted() == false. State clean between phases/topics.

  DETERMINISTIC_CHECK:
    - >= 10 tests exist and FAIL (TDD)
    - compile errors expected (stubs todo!())
    - grep -c "#\[test\]" csq_tests.rs >= 10

  ADVERSARIAL:
    - Hardcoded strings match stub defaults = false pass. Stubs use todo!()/unimplemented!().
    - T8 shared mutable counter without scoping = test isolation broken. Each test creates own BasicGrcEngine.

  ACCEPTANCE:
    - >= 10 tests in csq_tests.rs
    - all fail to compile or at runtime
    - zero timing-dependent tests

2D.4 IMPLEMENT: CSQ generation and inline feasibility logic
  PRE_GATE: 2D.3 complete; mcp_session_claim.
  TASK: Implement in BasicGrcEngine (reasoning/src/engine.rs):

    I1. CSQ generation (generate_csq):
        Input: function_id (name), sensitive_ops (sensitive API calls from caller).
        Logic:
          If sensitive_ops empty, return CsqResult { function_id, checks: vec![] } (passed() returns true since no Critical checks exist).
          For each sensitive_op, generate CsqCheck:
            Classify severity: file/network/money/auth patterns -> Critical; others -> Warning.
            Pattern matching (case-insensitive: lowercase both before comparison):
              CRITICAL patterns: "fs::", "file", "write", "delete", "remove", "net::", "tcpstream", "money", "payment", "auth", "token", "credential", "password"
              WARNING patterns: everything else in sensitive_ops.
            assumption: "The call to {sensitive_op} in {function_id} behaves as expected"
            consequence: severity-dependent template:
              Critical: "data loss/security breach/financial error possible if {sensitive_op} fails"
              Warning: "incorrect output possible if {sensitive_op} fails"
            guard_added: false (caller must set to true after adding guard code)
          Return CsqResult { function_id, checks }. passed() method computes: checks.iter().all(|c| c.severity != Critical || c.guard_added)

    I2. Inline feasibility check (check_inline_feasibility):
        Input: claim (new claim detected during EXECUTING), sources (known sources: PipelineSpec entries, DKG entries, etc.).
        Logic:
          Check InlineFeasibilityState.is_exhausted(). If yes, return Err(BudgetExhausted) with topic="inline_feasibility".
          Check if claim appears in any source (substring or exact match). Found: outcome = Pass.
          Not found: outcome = Fail (use plan's original approach).
          Blocked detection: if claim contains negation of plan assumption (heuristic: claim and source share >50% tokens but contradicting verb/predicate), outcome = Blocked.
          Note: heuristic intentionally simple for 2D. Phase 4F (FEASIBILITY_PROBE) adds full semantic analysis. For now, Blocked only returned when explicitly signaled via claim prefixed "CONTRADICTS_PLAN:".
          Record check: self.inline_feasibility_state.record_check().
          Return InlineFeasibilityResult { claim, method: FeasibilityMethod::SourceMatch, outcome }.

    I3. CsqResult.passed() method:
        Implement fn passed(&self) -> bool on CsqResult (not stored field).
        fn passed(&self) -> bool { self.checks.iter().all(|c| c.severity != Critical || c.guard_added) }
        Ensures consistent with current check state.
        No stored passed field — always computed.

  DETERMINISTIC_CHECK:
    - All 10 tests pass (0 failed)
    - No todo!()/unimplemented!() in CSQ/inline methods
    - No unwrap() in CSQ/inline implementation
    - All prior 2A-2C tests still pass

  ADVERSARIAL:
    - CRITICAL pattern list case-sensitive = "FS::" or "Net::" bypasses CSQ. Mixed-case unit test (e.g., "std::FS::write") still classified Critical. Compare lowercase.
    - Inline feasibility substring match too loose: "the" matches every source = false Pass. Match on meaningful content (>3 words or exact), not single common words.
    - Blocked false-positives on similar non-contradicting claims = replan loops. "uses std::fs::write" vs "uses std::fs::read" should NOT trigger Blocked.
    - InlineFeasibilityState shared across topics = exhausting in one phase blocks all subsequent. State resets between phases (reset exists and called by orchestrator).

  ACCEPTANCE:
    - cargo test: 0 failures (all 2A-2D)
    - cargo check: 0 errors
    - no todo!/unimplemented!/unwrap() in CSQ+inline methods
    - At least 10 tests pass in csq_tests.rs

2D.5 MEASURE: CSQ and inline feasibility metrics
  PRE_GATE: 2D.4 complete.
  TASK:
    - CSQ generation overhead per function (target <1ms for pattern matching)
    - Inline feasibility check overhead (target <2ms for source matching)
    - False positive rate on inline checks with synthetic claims:
        Prepare 20 claims: 10 present in sources, 10 absent.
        Assert: 0 false positives (present = Pass), 0 false negatives (absent = Fail). Pattern matching deterministic so rate should be 0%.
    - Record: test count delta from 2C, LOC delta, compile time delta
    - Write .elai_cc/metrics/phase_2D.toml
    - Update architecture-overview.txt s4 GrcEngine trait listing to include generate_csq and check_inline_feasibility (prevents spec drift)

  DETERMINISTIC_CHECK:
    - phase_2D.toml exists with csq_overhead_us, inline_check_overhead_us, false_positive_rate
    - csq_overhead_us < 1000
    - false_positive_rate = 0.0

  ADVERSARIAL:
    - Wall-clock time in CI = non-deterministic. std::hint::black_box + std::time::Instant for local only. Median of 1000 iterations. No criterion without approval. Instant for benchmarks acceptable under quarantine (metadata).
    - false_positive_rate uses trivial claims ("a", "b") = doesn't test realistic. Use realistic strings (>5 words each).

  ACCEPTANCE:
    - phase_2D.toml exists
    - csq_overhead_us < 1000
    - inline_check_overhead_us < 2000
    - false_positive_rate == 0.0
    - cargo test: 0 failures

PHASE 2D EXIT CRITERIA
  CSQ fires on every function with sensitive operations (file/network/money/auth).
  CSQ with Critical severity and no guard = failure (blocks progression).
  CSQ with Warning severity and no guard = warning only (does not block).
  Inline feasibility checks bounded at max 2 per EXECUTING phase.
  Inline feasibility Pass = proceed, Fail = use original plan, Blocked = replan.
  All 2D tests pass. All prior 2A-2C tests still pass (regression).
  Zero todo!/unimplemented!/unwrap() in CSQ and inline feasibility code.

PHASE 2 OVERALL: reasoning engine complete (2A state machine + 2B grounding verification + 2C per-agent config + 2D CSQ + inline feasibility).

PHASE 2D GROUNDING
  "CSQ: if [X] wrong, code will [Y]" -> GROUNDED reasoning.txt s6
  "Min 1 CSQ per file/network/money/auth function" -> GROUNDED reasoning.txt s6
  "Dangerous consequence = MUST add guard" -> GROUNDED reasoning.txt s6
  "Inline feasibility max 2 GRC calls" -> GROUNDED reasoning.txt s8
  "NEW claim not in PipelineSpec/Research Report/DKG triggers check" -> GROUNDED reasoning.txt s8
  "PASS->proceed, FAIL->original approach" -> GROUNDED reasoning.txt s8
  "Fundamentally different approach -> EXECUTING->BLOCK->PLANNING" -> GROUNDED reasoning.txt s8, orchestrator.txt s8 (MAJOR replan path)
  "Per-agent CSQ config: executor=mandatory, auditor=mandatory, compiler=none" -> GROUNDED reasoning.txt s7
  "CSQ severity classification (CRITICAL vs WARNING)" -> VERIFIED derived from reasoning.txt s6 "dangerous consequence" distinction. VeriGuard (OpenReview 2025) validates co-generation of verification with code.
  "Detection via function signature pattern matching" -> VERIFIED ast-grep (github.com/ast-grep/ast-grep) demonstrates tree-sitter structural matching. Our approach simpler (string patterns on caller-provided sensitive_ops list).
  "Runtime enforcement with bounded checks" -> VERIFIED AgentSpec (arxiv:2503.18666) implements DSL-based runtime enforcement with bounded safety. Our inline feasibility fixed-counter simplification.
  "InlineFeasibilityState separate from per-topic GRC budget" -> GROUNDED reasoning.txt s8 describes inline checks as cross-topic ("during EXECUTING"). Separate counter prevents cross-topic budget interference.
  "FeasibilityMethod enum instead of String for method field" -> GROUNDED reasoning.txt s10 L190, 194 explicitly require "method is VerificationMethod enum (not String)" for analogous fields.
  "<1ms CSQ overhead, <2ms inline check overhead" -> UNGROUNDED (verify empirically in 2D.5)
  "Blocked detection via CONTRADICTS_PLAN: prefix" -> UNGROUNDED simplified heuristic for 2D. Full semantic contradiction detection deferred to 4F (FEASIBILITY_PROBE). Prefix convention is pragmatic placeholder.
