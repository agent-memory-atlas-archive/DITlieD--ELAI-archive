PHASE 2B AUDIT — GROUNDING VERIFICATION
========================================
Auditor: hostile implementation auditor
Target: F:\.ELAI_workflow\.elai_cc\crates\reasoning\
Spec: F:\.ELAI_workflow\docs\thefinalglance\summary\phase-2-summary.md (lines 331-739)
Map: F:\.ELAI_workflow\.claude\tools\elai.map
Date: 2026-04-24

OVERALL VERDICT: PASS WITH MINOR DEVIATIONS
Type-level surface area, six-method enum, breaking trait signatures, 16 tests, and the
plan-sweep algorithm are all present and functional. Two material deviations from the
spec text: (1) the `tools/` crate dependency was NOT added — reasoning/ uses an internal
FileReader trait (StdFileReader) instead, decoupling without dependency; (2) the
EXECUTING-block enforcement is split across two layers — reasoning/ exposes
PlanGroundingResult.all_grounded, but the orchestrator GroundingSweepGate consumes a
separately-populated TransitionContext.ungrounded_claims field rather than invoking
engine.ground_plan(). Neither deviation breaks Phase 2B's contract; both should be
flagged for Phase 2C/3A reconciliation.

================================================================
PER-SUBSTEP VERDICTS
================================================================

----------------------------------------------------------------
2B.1 SKELETON: types + trait extension
----------------------------------------------------------------
VERDICT: PASS (with deviation D-1)

EVIDENCE:
  VerificationMethod enum, 6 variants, derives Debug/Clone/Copy/PartialEq/Eq/Hash:
    types.rs:311-319 — DEF_ENUM ApiCheck, ApiRegistryCheck, FileCheck, SourceCheck,
    TestCheck, WebCheck. Also derives serde::Serialize/Deserialize (extension,
    not regression).
  requires_network() and requires_tools(): types.rs:321-341. Network = Web/Api/
    ApiRegistry; tools = File/Source/Test. Matches spec exactly.
  GroundingResult { claim, method: VerificationMethod, level, source, verified_at }:
    types.rs:359-368. verified_at: u64. Method field IS the enum (not String) —
    adversarial check passes.
  VerificationResult { claim, passes, evidence, method_used }: types.rs:347-353.
  PlanGroundingResult { claims, all_grounded, ungrounded, verify_needed }:
    types.rs:374-384. all_grounded semantics documented as
    "True iff ungrounded.is_empty()" — matches spec semantics.
  GrcError::VerificationError { method, claim, reason }: types.rs:161-166.
  Verifier trait Send + Sync, supported_methods() + verify(): verification.rs:12-21.
  Object-safety asserted at compile time: verification.rs:359-361
    `const _: () = { fn _assert_object_safe(_: &dyn Verifier) {} };`
  TestRunner trait Send + Sync, run_test(name, crate_name) -> TestRunResult:
    verification.rs:29-31. TestRunResult { passed, output, duration_ms }:
    types.rs:390-395.
  MockTestRunner: verification.rs:51-82. Implements TestRunner.
  CargoTestRunner: NOT present in reasoning/. Spec D8 stipulates Production
    runner uses BashSandbox via tools/. It belongs in tools/, not reasoning/.
    NoopTestRunner (verification.rs:157-179) acts as the default fallback.
    DEVIATION D-1 (acceptable): CargoTestRunner deferred. The DI seam exists
    (LocalVerifier::new takes Box<dyn TestRunner>); production wiring will
    plug the real runner from a downstream crate.
  GrcEngine trait stub method signatures: engine.rs:24-65.
    verify_claim(&self, claim, source, context) -> Result<VerificationResult, GrcError>
      — 3-arg signature confirmed (D7 BREAKING change present).
    ground_plan(&self, plan) -> PlanGroundingResult
      — return type changed (D6 BREAKING change present).
  LocalVerifier::new takes Box<dyn FileReader> + Box<dyn TestRunner>:
    verification.rs:191-198. Spec called for Box<dyn TestRunner> only with FILE
    operations going through tools::read_file directly. Implementation uses an
    abstracted FileReader trait (verification.rs:39-45) — DEVIATION D-2 (see
    "tools/ dep" finding).

----------------------------------------------------------------
2B.2 VERIFY: compilation + dependency check
----------------------------------------------------------------
VERDICT: PASS (with deviation D-2 noted)

EVIDENCE:
  Cargo.toml deps: elai-core, thiserror, serde, regex (Cargo.toml:8-13).
  No `tools` dependency added.
  DEVIATION D-2 (material): The spec (2B.1 task line 469, 2B.2 line 506,
  GROUNDING SUMMARY line 731) explicitly directed adding `tools/` as a
  reasoning/ dependency, with FILE_CHECK and SOURCE_CHECK calling
  tools::read_file. Implementation chose a different design: a FileReader
  trait inside reasoning/ with StdFileReader (real fs) and MockFileReader
  (test) implementations. This is arguably BETTER — it preserves dependency
  inversion, avoids the new coupling that the spec itself flagged as a risk
  (R-2B-1), and keeps the spec-flagged adversarial concern about tools/ ->
  knowledge/ transitive deps moot. However, it diverges from the literal
  spec instruction and from GROUNDING SUMMARY's "tools/ as reasoning/
  dependency" claim. Recommendation: ratify the chosen design in 2C planning
  notes; do NOT retro-add the dependency.
  No circular dep risk (no `tools` dependency exists at all).
  `regex = "1"` is an UNDOCUMENTED new dependency relative to the spec list
  (core, tools, thiserror, serde). Required for ground_plan claim regex
  (engine.rs:501) and SOURCE_CHECK pattern matching (verification.rs:229).
  Acceptable; flag for visibility.

----------------------------------------------------------------
2B.3 TEST: 16 failing TDD tests
----------------------------------------------------------------
VERDICT: PASS

EVIDENCE:
  tests/verification_tests.rs: `grep -c "#\[test\]"` -> 16. Exactly the
  required count.
  T1 file_check_existing_file: lines 16-34. Uses tempfile::NamedTempFile.
  T2 file_check_nonexistent_file: lines 41-55.
  T3 source_check_function_exists: lines 62-80. Asserts evidence contains
    function name.
  T4 source_check_function_missing: lines 87-101.
  T5 test_check_passing_test: lines 108-125. Uses MockTestRunner — no
    subprocess spawn (R-2B-2 mitigated).
  T6 web_check_stub_returns_error: lines 132-144. Asserts
    GrcError::VerificationError, method == WebCheck.
  T7 api_check_stub_returns_error: lines 151-163.
  T8 api_registry_check_stub_returns_error: lines 170-182.
  T9 plan_grounding_sweep_blocks_ungrounded: lines 190-201. Plan with
    "Grounding: file:src/main.rs" and "Grounding: file:nonexistent.rs".
    Asserts !all_grounded and ungrounded contains the missing claim.
  T10 plan_grounding_sweep_passes_all_grounded: lines 209-218. Uses
    "Grounding: file:Cargo.toml" — relies on cwd being crate root.
  T11 plan_grounding_sweep_empty_plan: lines 227-234. R-2B-3 covered.
  T12 uncertain_claim_auto_resolution: lines 244-260.
  T13 verification_method_requires_network: lines 268-293. Pure
    compile-time/enum check.
  T14 verifier_trait_object_safety: lines 301-305. Box<dyn Verifier>
    constructed with LocalVerifier — confirms object safety at runtime
    in addition to the compile-time assertion in verification.rs:359-361.
  T15 plan_sweep_adversarial_regex: lines 315-331. Comment containing
    "// see file:fake_path" must NOT be extracted as a claim. The regex
    in engine.rs:501 anchors on the literal "Grounding:" prefix, so this
    is correctly handled.
  T16 verify_claim_strips_prefix: lines 340-354. Asserts the claim field
    in the returned VerificationResult equals "src/main.rs", not
    "file:src/main.rs". This validates I3's BLOCK fix.

  No network calls in test code:
    grep -c "reqwest\|hyper\|TcpStream\|http://" -> 0.
  No timing-dependent code:
    grep finds no sleep/Duration/Instant/SystemTime in test file.
  Temp files use tempfile crate (T1, T3, T4, T12).
  TDD contract: tests existed and failed against the 2A stubs. By 2B.4
  they pass.

----------------------------------------------------------------
2B.4 IMPLEMENT: verification methods + plan sweep
----------------------------------------------------------------
VERDICT: PASS (with sub-finding F-1 below)

EVIDENCE — LocalVerifier (verification.rs:200-300):
  FILE_CHECK (verification.rs:208-220): file_reader.file_exists(claim) ->
    VerificationResult { passes, evidence: "file exists: {claim}" or
    "file not found: {claim}", method_used: FileCheck }. Spec wanted
    evidence "File exists: {path}, {total_lines} lines" — implementation
    omits line count. Spec deviation cosmetic; tests T1/T2 only assert
    the path appears in evidence and don't require the line count.
    SUB-FINDING F-1 (cosmetic).
  SOURCE_CHECK (verification.rs:221-264): builds regex
    r"(fn|struct|enum|trait|impl|const|static|type|mod)\s+{escaped}\b"
    EXACTLY matching the spec at line 600-601. Uses regex::escape on the
    symbol — protects against regex injection (better than spec). Walks
    contents.lines() to find the first match, captures kind + line number,
    formats "Found {kind} {claim} at {file}:{line}". I/O failure returns
    passes=false with diagnostic, not Err — matches spec's intent.
  TEST_CHECK (verification.rs:265-284): delegates to injected TestRunner.
    Returns passes from result.passed; evidence includes duration_ms on
    success. NOT wired to BashSandbox in reasoning/ (see 2B.1 D-1
    deviation: production CargoTestRunner is downstream).
  Default branch (verification.rs:285-289): returns
    GrcError::VerificationError for any non-Local method passed to
    LocalVerifier — defensive layering.
  supported_methods returns [FileCheck, SourceCheck, TestCheck]
    (verification.rs:293-299).

EVIDENCE — NetworkVerifier (verification.rs:306-354):
  Stubbed exactly per spec I2: WebCheck, ApiCheck, ApiRegistryCheck all
  return Err(GrcError::VerificationError { method, claim, reason: "network
  verification not yet implemented" }). NO panic/todo!/unimplemented!
  (DETERMINISTIC_CHECK passes).
  supported_methods returns [WebCheck, ApiCheck, ApiRegistryCheck].

EVIDENCE — verify_claim (engine.rs:471-489):
  Parses prefix via parse_claim_prefix (engine.rs:147-165):
    file: -> FileCheck (strip)
    grep: -> FileCheck (strip)
    test: -> TestCheck (strip)
    search: -> WebCheck (strip)
    tool: -> SourceCheck (strip)
    training: -> FileCheck (auto-resolution attempt)
    no prefix -> FileCheck with full claim (defensive default)
  Spec mapping matches exactly. Falls back to source as ctx if context is
  empty (helpful relaxation). Result.claim is overwritten to the stripped
  reference (engine.rs:486-487) — validates T16.

EVIDENCE — ground_plan (engine.rs:491-579):
  Empty/whitespace plan returns trivially grounded (engine.rs:492-499) —
  R-2B-3 satisfied.
  Regex anchored on "Grounding:" prefix at line start
  (engine.rs:501): r"(?m)^(?:\s*Grounding:\s*)((file|grep|test|search|tool|
  training):[^\s]+)". Adversarial T15 (comment with "file:fake_path")
  excluded by anchor — verified.
  For each captured claim:
    training: tries auto-resolution via FileCheck on the stripped path; if
      passes, appended to claims with level=Verified; else added to BOTH
      verify_needed AND ungrounded with level=Uncertain. Matches spec
      semantics that verify_needed is a SUBSET of ungrounded
      (types.rs:381-383 doc).
    other: dispatch to verify_claim; if passes -> Verified, else added to
      ungrounded with Uncertain.
  Constructs PlanGroundingResult with all_grounded = ungrounded.is_empty().
  No todo!/unimplemented!/unwrap()/panic! in verification.rs (verified by
  grep).
  No .unwrap() in verification.rs (verified). engine.rs ground_plan does
  use `.expect("claim regex is valid")` on a static regex at line 502 —
  acceptable per project rules (provably valid literal at compile time).

EVIDENCE — UNCERTAIN auto-resolution (I5):
  Implemented inline within ground_plan for "training:" claims
  (engine.rs:512-538). For ThinkStep-driven path, parse_grounding still
  classifies "training:" as Uncertain and emits a warning
  (engine.rs:391-396). The think() path does NOT call FILE_CHECK to
  promote — that auto-resolution happens only during ground_plan. This is
  a partial implementation: the spec sentence "Called during think() for
  steps with Uncertain grounding (enhances 2A logic)" is NOT realized in
  think(). Severity: low — the plan-sweep path is the binding decision
  point for EXECUTING block; the in-step warning is sufficient
  observability.

----------------------------------------------------------------
2B.5 MEASURE: performance metrics
----------------------------------------------------------------
VERDICT: PASS

EVIDENCE — F:\.ELAI_workflow\.elai_cc\metrics\phase_2B.toml:
  file_check_latency_us = 0 (ns = 100, p99 = 200 ns)
    Target: <2000 us. PASS by ~4 orders of magnitude (mock-based, real
    fs would be slower but well under threshold).
  source_check_latency_us = 201 (ns = 201,150, p99 = 321,400 ns)
    source_check_file_lines = 1000 (adversarial requirement met).
    Target: <10,000 us. PASS.
  plan_sweep_latency_us = 92 (ns = 92,800, p99 = 188,100 ns)
    plan_sweep_claim_count = 20 (adversarial requirement met).
    Target: <100,000 us. PASS.
  test_count = 34 (verification_tests.rs has 16; remaining 18 are
    grc_engine.rs / agent_config.rs / csq_tests.rs aggregated by the
    metrics harness — not phase 2B-specific but reported together).
  test_failures = 0.
  methods_implemented = 3, methods_stubbed = 3, methods_total = 6.
  implemented = ["FILE_CHECK", "SOURCE_CHECK", "TEST_CHECK"].
  stubbed = ["WEB_CHECK", "API_CHECK", "API_REGISTRY_CHECK"].
  Compile time delta from 2A baseline: +0.55s (2A was 0.20s -> 2B is 0.75s).
  Bench harness exists at crates/reasoning/benches/verification_bench.rs
  (elai.map line 39680: bench_ground_plan; lines 39524, 39682 for callsite
  resolution).

================================================================
WIRING CHECK
================================================================
Question: who calls verify_claim and ground_plan?

  reasoning/src/engine.rs:541 — ground_plan internally calls verify_claim
    on each non-training claim. Self-consistent loop.

  orchestrator/src/grc.rs:92-100 — PipelineGrc::verify_claim wraps
    self.engine.verify_claim(claim, source, context) and lifts errors
    through PipelineGrcError::Engine.
  orchestrator/src/grc.rs:108-110 — PipelineGrc::ground_plan wraps
    self.engine.ground_plan(plan).
  orchestrator/src/grc.rs:264-268, 272-276 — unit tests confirm dispatch.
  orchestrator/src/grc.rs:7, 42 — module doc explicitly identifies
    verify_claim/ground_plan as "Integration point 2: plan evaluation /
    claim grounding".

  benches/verification_bench.rs:82, 106, 142 — the verification benchmarks
    drive verify_claim and ground_plan via BasicGrcEngine::with_verifier.

  Beyond PipelineGrc, NO production code path in the workspace currently
  invokes ground_plan or verify_claim against a real plan. The orchestrator
  exposes the API but nothing in the FSM transition machinery calls it.
  Search-and-replace surveyed: only test files and benches.

WIRED?: PARTIAL. Type-level wiring is in place (PipelineGrc -> BasicGrcEngine).
Behavioural wiring (FSM gate -> ground_plan) is NOT in place.

================================================================
SWEEP BLOCKING ENFORCEMENT
================================================================
Question: does anything block the EXECUTING transition when ungrounded
claims exist?

  orchestrator/src/evidence.rs:191-218 — GroundingSweepGate exists and
    implements EvidenceGate.
  orchestrator/src/evidence.rs:208 — `match &ctx.ungrounded_claims` is the
    blocking check. The gate consumes a pre-populated
    TransitionContext.ungrounded_claims: Option<Vec<String>>
    (evidence.rs:108).
  orchestrator/src/evidence.rs:660-682 — three unit tests:
    test_gate2_all_grounded_passes (empty vec -> Ok)
    test_gate2_ungrounded_claims_blocks (non-empty vec -> Err with
      missing_artifacts == claims)
    test_gate2_missing_report_blocks (None -> Err: missing
      "grounding_sweep_report")
  orchestrator/src/evidence.rs:471 — gate is registered in the gate list
    (Box::new(GroundingSweepGate)).

GAP: GroundingSweepGate does NOT call engine.ground_plan() to populate
ungrounded_claims. The gate enforces blocking IF something else has
already produced the list. Today, nothing in the orchestrator pipeline
calls ground_plan() to populate it. The plumbing
  PlanGroundingResult.ungrounded -> TransitionContext.ungrounded_claims
is missing.

SWEEP BLOCKING ENFORCED?: STRUCTURALLY YES, OPERATIONALLY NO. The gate
exists and would block correctly if fed; the producer side that calls
ground_plan() and populates ctx.ungrounded_claims is absent in current
code.

This is consistent with the spec's forward-dependency note (line 349-351):
"Phase 4A changes verification execution model — agents proxy verification
via IPC GroundingRequest messages." Phase 2B is permitted to land the
verification primitives without driving them from the FSM. Phase 2C/3A/4A
must close the loop.

================================================================
BREAKING CHANGE VERIFICATION (D6, D7)
================================================================

D6. ground_plan return type:
  Phase 2A stub: fn ground_plan(&self, plan: &str) -> Vec<GroundingResult>
  Phase 2B impl: fn ground_plan(&self, plan: &str) -> PlanGroundingResult
  Confirmed at engine.rs:38 (trait) and engine.rs:491 (impl).
  PlanGroundingResult exists in types.rs:374-384 with fields claims,
  all_grounded, ungrounded, verify_needed.
  BREAKING CHANGE: PRESENT.

D7. verify_claim arity:
  Phase 2A stub: fn verify_claim(&self, claim, source) -> ... (2 args)
  Phase 2B impl: fn verify_claim(&self, claim, source, context) -> ...
  Confirmed at engine.rs:32-37 (trait) and engine.rs:471-476 (impl).
  Third parameter `context: &str` is plumbed into the Verifier dispatch.
  BREAKING CHANGE: PRESENT.

================================================================
ENUM SHAPE CHECK
================================================================
VerificationMethod (types.rs:311-319): EXACTLY 6 variants
  FileCheck, SourceCheck, TestCheck, WebCheck, ApiCheck, ApiRegistryCheck.
Cross-checked via elai.map DEF_ENUM rows (lines 24901, 25166 + 4 more).
Spec EXIT CRITERIA satisfied.

================================================================
WHICH 3 ARE STUBS, AND HOW STUBS BEHAVE
================================================================
Stubs: WEB_CHECK, API_CHECK, API_REGISTRY_CHECK.
Implementer: NetworkVerifier (verification.rs:306-354).
Behavior: returns Err(GrcError::VerificationError { method, claim, reason:
"network verification not yet implemented" }). Does NOT panic, does NOT
return passes=false silently, does NOT swallow errors. Caller can match
on VerificationError to detect "stubbed method" without having to inspect
strings — but currently the only signal is the reason string. Spec
acceptance ("Network stubs return typed errors (not panics)") satisfied.

================================================================
ZERO-LAZY-CODE CHECK
================================================================
verification.rs: 0 occurrences of `todo!`, `unimplemented!`, `panic!`.
verification.rs: 0 occurrences of `.unwrap()`.
engine.rs verify_claim block (lines 471-489): 0 todo!/unimplemented!.
engine.rs ground_plan block (lines 491-579): 1 `.expect("claim regex is
  valid")` on a static regex compile (line 502). Acceptable per project
  rule (const/static init with provably valid literal).
No `pass` bodies, mock data, or "# rest" placeholders.

================================================================
EXIT CRITERIA RE-CHECK
================================================================
[X] All 6 verification methods have type-level representation
[X] 3 methods fully implemented (FILE/SOURCE/TEST)
[X] 3 methods stubbed with typed errors (WEB/API/API_REGISTRY)
[X] verify_claim dispatches to correct verifier based on grounding format
[X] ground_plan decomposes plan into claims, verifies each, returns
    PlanGroundingResult
[X] Ungrounded claims detected and reported (all_grounded == false when
    any claim fails)
[~] UNCERTAIN grounding auto-resolved via FILE_CHECK/SOURCE_CHECK where
    possible — happens in ground_plan, NOT in think() (partial; see 2B.4
    sub-finding)
[X] Empty plan handled (trivially grounded, no panic)
[X] All 16+ tests pass; metrics report 0 failures across 34 tests
[X] Zero todo!/unimplemented!/unwrap()/panic! in verification code
[X] No network calls in any test (grep verified)

================================================================
SUMMARY OF FINDINGS
================================================================
PASS: type system, 6 enum variants, breaking signatures, 16 tests, plan
sweep regex anchoring, training-claim auto-resolution, performance
benchmarks under target, no lazy code, no panic surface in stubs.

DEVIATIONS (acceptable, flag for downstream phases):
  D-1. CargoTestRunner not present in reasoning/. Belongs in tools/ or
       orchestrator/. DI seam is correct.
  D-2. tools/ crate dependency NOT added. Implementation chose a
       reasoning/-internal FileReader trait instead. Better design but
       diverges from literal spec. Update GROUNDING SUMMARY for 2C.

PARTIAL:
  P-1. UNCERTAIN auto-resolution does NOT run inside think(); only in
       ground_plan. Spec line 646 stated "Called during think() for steps
       with Uncertain grounding". Severity LOW.
  P-2. Sweep blocking gate (GroundingSweepGate) exists and blocks correctly
       given a pre-populated TransitionContext.ungrounded_claims, but no
       production code path calls ground_plan() to populate it. The
       reasoning -> orchestrator -> FSM wiring is structurally complete,
       behaviorally incomplete. Phase 4A is the natural place to close
       this loop per the forward-dependency note.

COSMETIC:
  F-1. FILE_CHECK evidence string omits line count ("File exists: {path},
       {total_lines} lines" per spec) — uses "file exists: {path}" only.

NEW DEPENDENCY (flag for visibility):
  regex = "1" added to Cargo.toml. Required for ground_plan claim
  extraction and SOURCE_CHECK pattern matching. Not in spec dependency
  list (spec listed core, tools, thiserror, serde).

OVERALL: Phase 2B's contract is honored. The 6 methods are present in
type space, the 3 implemented methods work end-to-end against tests, the
3 stubs return typed errors as required, the breaking signatures are in
place, the plan sweep produces accurate PlanGroundingResult outputs, and
the metrics file is populated with passing latency targets. The two
material gaps (tools/ dep substitution; FSM-side wiring of
ungrounded_claims) are design choices/deferred plumbing, not contract
violations.
