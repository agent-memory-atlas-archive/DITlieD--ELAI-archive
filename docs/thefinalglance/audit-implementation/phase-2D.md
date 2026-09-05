PHASE 2D AUDIT — CSQ AND INLINE FEASIBILITY
============================================
Auditor mode: HOSTILE. Evidence-only. Every verdict cites file:line.
Spec source: docs/thefinalglance/summary/phase-2-summary.md L1111-1460
Target: .elai_cc/crates/reasoning/

================================================================
2D.1 SKELETON — CSQ + inline feasibility types and stubs
================================================================
VERDICT: IMPLEMENTED.

Evidence (types in crates/reasoning/src/types.rs):
- CsqSeverity enum { Critical, Warning } at L404-408. Derives include
  Debug/Clone/Copy/PartialEq/Eq as required.
- CsqCheck { assumption, consequence, severity, guard_added } at L413-419.
  guard_added: bool field is plain — no Default impl coerces it true (adversarial
  block satisfied).
- CsqResult { function_id, checks } at L423-427. passed() method at L432-436
  computed dynamically: `checks.iter().all(|c| c.severity != Critical || c.guard_added)`.
  No stored passed field.
- FeasibilityOutcome enum { Pass, Fail, Blocked } at L446-450.
- FeasibilityMethod enum { SourceMatch } at L455-457.
- InlineFeasibilityResult { claim, method, outcome } at L461-465.
- InlineFeasibilityState { checks_used: u32, max_checks: u32 } at L470-474.
  max_checks field is PRIVATE; Default sets max_checks=2 at L476-483
  (adversarial block on u32::MAX abuse satisfied — no public constructor lets
  callers pick max_checks). Methods is_exhausted/record_check/reset/max_checks
  at L485-501. record_check uses saturating_add (overflow safe).

Evidence (trait + struct in crates/reasoning/src/engine.rs):
- GrcEngine::generate_csq trait method declared at L51-55.
- GrcEngine::check_inline_feasibility (&mut self) at L60-64.
- BasicGrcEngine struct gains `pub inline_feasibility_state:
  InlineFeasibilityState` at L75. new()/with_config initialize via
  Default::default() at L86 and L97.

Evidence (re-exports in crates/reasoning/src/lib.rs L10-15):
- All 7 new types re-exported: CsqCheck, CsqResult, CsqSeverity,
  FeasibilityMethod, FeasibilityOutcome, InlineFeasibilityResult,
  InlineFeasibilityState.

Notes:
- Spec asked for `Default: max_checks = 2`. Confirmed.
- Spec asked: "Stubs in BasicGrcEngine using todo!() until step 2D.4".
  Current code has full implementations (this is post-2D.4 state). Acceptable
  because this audit verifies the END state of 2D.

================================================================
2D.2 VERIFY — compilation
================================================================
VERDICT: NOT INDEPENDENTLY RE-RUN BY THIS AUDITOR (would require cargo).
Static evidence supports compile-clean assertion:
- All trait methods have impl blocks in BasicGrcEngine (engine.rs L591 and L649).
- No `todo!()`/`unimplemented!()` in generate_csq body (L591-647) or
  check_inline_feasibility body (L649-698). Verified by reading.
- No `.unwrap()` in either method body. Verified by reading.
- Types re-exported (lib.rs L10-15) match call sites in tests and orchestrator.
Recommend: auditor agent run `cargo test -p reasoning --test csq_tests` for
final confirmation.

================================================================
2D.3 TEST — 10 failing tests (TDD)
================================================================
VERDICT: TESTS EXIST AND ARE WIRED. Final state — they PASS (which is the 2D.4
goal). Count and content match spec.

Evidence (crates/reasoning/tests/csq_tests.rs):
- T1 csq_generates_check_for_file_function — L10-27. generate_csq("write_config",
  &["std::fs::write"]) -> checks non-empty, at least one Critical. Matches spec.
- T2 csq_critical_without_guard_fails — L33-48. Critical+guard_added=false ->
  passed()==false.
- T3 csq_critical_with_guard_passes — L53-65. Critical+guard_added=true ->
  passed()==true.
- T4 csq_warning_without_guard_still_passes — L70-85. Warning+no guard ->
  passed()==true.
- T5 csq_no_sensitive_ops_returns_empty — L90-104. Empty ops -> empty checks
  -> passed()==true.
- T6 inline_feasibility_pass_on_known_claim — L110-124. Multi-word claim, source
  contains it -> Pass. Avoids trivial-substring false positive (adversarial
  block satisfied).
- T7 inline_feasibility_fail_on_unknown_claim — L129-140. Empty sources -> Fail.
- T8 inline_feasibility_max_two_calls — L147-180. Initial 0; 2 OK; 3rd returns
  Err(BudgetExhausted{..}). Verifies engine.inline_feasibility_state.checks_used
  observable.
- T9 inline_feasibility_blocked_signals_replan — L188-202. Claim with
  "CONTRADICTS_PLAN:" prefix -> Blocked.
- T10 inline_feasibility_state_reset — L208-224. After 2 checks exhausted; reset
  -> 0 / not exhausted.

Test count: 10. Matches spec exactly. `#[test]` attribute present on each.
Each test instantiates its own BasicGrcEngine — isolation preserved
(adversarial block satisfied).

ADDITIONAL TESTS (orchestrator-side):
crates/orchestrator/src/grc.rs has 6 PipelineGrc tests at L221-281, including
generate_csq_empty_ops, generate_csq_file_write_critical,
check_inline_feasibility_pass_when_claim_in_sources, and
check_inline_feasibility_exhausted_after_two_calls. Bonus coverage at the
wrapper layer.

================================================================
2D.4 IMPLEMENT — CSQ generation and inline feasibility logic
================================================================
VERDICT: IMPLEMENTED CORRECTLY per spec I1/I2/I3.

I1 — generate_csq (engine.rs L591-647):
- Empty sensitive_ops short-circuits to `CsqResult { function_id, checks: vec![] }`
  (L596-601). Matches spec.
- Critical pattern list at L603-606: `"fs::", "file", "write", "delete",
  "remove", "net::", "tcpstream", "money", "payment", "auth", "token",
  "credential", "password"`. Matches spec D2 keyword set (data_loss/security/
  financial/auth coverage). Includes BOTH the user-listed CRITICAL keywords
  (data_loss via file/write/delete; security via auth/token/credential/password;
  financial via money/payment; auth_bypass via auth/token/credential/password).
- Case-insensitive: `op_lower = op.to_lowercase()` at L611, pattern check
  `op_lower.contains(pat)` at L612-614. Adversarial block on "FS::"/"Net::"
  case-bypass satisfied.
- Severity classification: Critical if any pattern matches, else Warning
  (L616-620). Matches spec.
- Consequence templates (L622-629) match spec wording exactly.
- assumption template (L632-635) matches spec.
- guard_added: false default at L638. Caller must explicitly set true.

I2 — check_inline_feasibility (engine.rs L649-698):
- Budget exhaustion check at L655-661 returns
  Err(GrcError::BudgetExhausted { topic: "inline_feasibility", calls_used,
  max_calls }). Matches spec.
- Blocked detection at L665-673 only when claim begins with
  `CONTRADICTS_PLAN:` prefix and trimmed remainder is non-empty. Matches I2
  exactly: "Blocked only when explicitly signaled by caller via claim prefixed
  CONTRADICTS_PLAN:". Phase 4F semantic detection deferred — correct.
- Source-match path (L674-688) checks `source.contains(claim_trimmed)`.
  Pass if any source contains; else Fail.
- record_check at L691 increments counter AFTER outcome decision (correct order).
- Returns InlineFeasibilityResult with method = SourceMatch (L693-697).

I3 — CsqResult.passed() (types.rs L432-436): Computed, not stored. Empty checks
list returns true (vacuous all). Critical+!guard returns false. Verified.

Adversarial blocks satisfied:
- Mixed-case "std::FS::write" -> Critical (lowercase match).
- Substring loose-match risk mitigated by tests using meaningful multi-word
  claims (T6, T8 use unique strings).
- Blocked false positive prevented — only explicit CONTRADICTS_PLAN: prefix
  triggers it. "uses std::fs::write" vs "uses std::fs::read" cannot
  accidentally trigger Blocked (needs prefix).
- State sharing across phases handled by InlineFeasibilityState::reset() at
  types.rs L494-496. Spec assigns reset responsibility to orchestrator
  between phases — exposed but NOT YET WIRED (see "Wiring" below).

================================================================
2D.5 MEASURE — metrics
================================================================
VERDICT: IMPLEMENTED.

Evidence (.elai_cc/metrics/phase_2D.toml):
- csq_overhead_us = 0 (sub-microsecond median; csq_overhead_ns = 900) — well
  under 1000us target.
- inline_check_overhead_us = 1 (1400ns median) — well under 2000us target.
- false_positive_rate = 0.0 / false_negative_rate = 0.0 over 20 claims
  (10 present, 10 absent). Matches spec.
- test_count = 62, test_failures = 0.
- p99 percentiles also recorded.

Bench source: crates/reasoning/tests/csq_bench.rs L14-50.
Method: std::hint::black_box + std::time::Instant, median of 1000 iterations,
local timing only — no criterion dep introduced. Adversarial block satisfied.

Note: spec also asked to update architecture-overview.txt section 4 GrcEngine
trait listing. NOT VERIFIED IN THIS AUDIT — out of crate scope but flagged for
follow-up.

================================================================
EXIT CRITERIA CHECK (Phase 2D)
================================================================
[X] CSQ fires on every function with sensitive operations — generate_csq
    classifies via pattern set covering file/network/money/auth.
[X] CSQ Critical + no guard = failure that blocks — passed() returns false.
    BUT: nothing in the orchestrator yet HONOURS this verdict (see Wiring).
[X] CSQ Warning + no guard = warning only — passed() ignores Warning severity.
[X] Inline feasibility bounded at 2 per EXECUTING phase — enforced by
    InlineFeasibilityState; third call returns BudgetExhausted.
[X] Pass / Fail / Blocked outcomes — FeasibilityOutcome enum covers all three.
[X] All 10 2D tests pass (per metrics test_failures=0).
[X] Zero todo!()/unimplemented!()/unwrap() in CSQ + inline impl bodies — verified
    by reading L591-698.

================================================================
WIRING AUDIT — IS CSQ ACTUALLY ENFORCED?
================================================================
VERDICT: PARTIAL. Engine + orchestrator wrapper exist. Production callers DO NOT.

PipelineGrc wrapper (crates/orchestrator/src/grc.rs):
- L122-130: PipelineGrc::generate_csq forwards to engine. Documented as
  "driven by pre-tool-call hooks for sensitive operations".
- L141-149: PipelineGrc::check_inline_feasibility forwards to engine.
  Documented as "driven by EXECUTING phase transition handlers".
- Re-exported from orchestrator lib.rs L347.

Production wiring search results (Grep across .elai_cc):
- generate_csq callers: ONLY tests (csq_tests.rs T1/T5, csq_bench.rs L28,
  PipelineGrc tests at grc.rs L224/L233). ZERO production call sites.
- check_inline_feasibility callers: ONLY tests (csq_tests.rs T6-T9,
  csq_bench.rs L70/L149/L161, PipelineGrc tests at grc.rs L244/L254-256).
  ZERO production call sites.
- file_ops.rs, dispatcher.rs, fsm.rs, mcp_tool_bridge.rs, permissions.rs,
  permission_enforcer.rs, gate.rs, autonomous_gate.rs, deterministic_audit_gate.rs
  — none import or call PipelineGrc::generate_csq or
  PipelineGrc::check_inline_feasibility.

CONSEQUENCE:
A Critical CSQ failure today is an in-memory bool that no enforcement path
reads. The CSQ engine cannot block a Write/Edit/Bash because no pre-tool
hook invokes it. Inline feasibility's Blocked outcome cannot trigger the
spec'd EXECUTING->PLANNING transition because no EXECUTING handler invokes
it (and reset() between phases is similarly orphaned).

This is consistent with the spec's PHASE BOUNDARY: phase 2D delivers the
REASONING ENGINE; orchestrator-side wiring (pre-tool hook calling
generate_csq, EXECUTING handler calling check_inline_feasibility, replan
emission on Blocked) belongs to phase 3A+. The spec D4 explicitly notes
"actual ReplanRequest emission is orchestrator's responsibility (3A+)".

Therefore: 2D as specified is COMPLETE. Cross-phase enforcement is NOT
present yet, by design.

ReplanRequest emission path (spec asked auditor to verify):
- No `ReplanRequest` symbol found anywhere consuming Blocked outcome.
- This is correct for phase 2D — the spec scopes only the Blocked outcome
  return value to 2D. ReplanRequest plumbing is a 3A+ deliverable.

================================================================
SUMMARY
================================================================
2D.1 SKELETON ............ PASS  (7 types + 2 trait methods + state field)
2D.2 VERIFY .............. PASS  (static reading; no recompile run by auditor)
2D.3 TEST ................ PASS  (10 tests in csq_tests.rs, plus 4 wrapper tests)
2D.4 IMPLEMENT ........... PASS  (logic matches I1/I2/I3 with adversarial
                                   protections)
2D.5 MEASURE ............. PASS  (metrics file present, all targets met)
EXIT CRITERIA ............ PASS  (all 7 met within crate scope)

CSQ ENFORCEMENT WIRED ........... NO — engine + wrapper exist, no production
                                  caller before destructive tool ops. Out of
                                  2D scope per spec D4 (3A+).
FEASIBILITY ENFORCEMENT WIRED ... NO — engine + wrapper exist, no EXECUTING
                                  handler invokes it; no ReplanRequest
                                  emitter. Out of 2D scope per spec D4.

OVERALL VERDICT: PHASE 2D IMPLEMENTATION COMPLETE.
Phase 3A+ MUST add: (a) pre-Write/Edit/Bash hook calling
PipelineGrc::generate_csq with the function's sensitive_ops list and BLOCKING
when result.passed()==false; (b) EXECUTING-phase handler calling
PipelineGrc::check_inline_feasibility on novel claims and emitting
ReplanRequest(MAJOR) on Blocked; (c) inline_feasibility_state.reset() at
phase boundaries.

Follow-up flags:
- architecture-overview.txt section 4 update (spec 2D.5 task) NOT verified —
  document drift risk.
- Add a smoke test (orchestrator-level) once wiring lands to prove that a
  Critical CSQ without guard truly blocks a tool call. Until then, the
  enforcement contract is asserted by tests but not by the production code
  path.
