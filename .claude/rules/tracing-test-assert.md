tracing-test mandatory-assert (WIRING-FIX-ELAI Phase 3 / SP4, HARD RULE, fail-closed)

Loaded by: any task that authors or audits a Rust test using the tracing-test crate (#[traced_test], logs_contain, logs_assert), or any Layer-4 runtime trace-assertion harness. Enforced by the auditor, not the main LLM.

WHY THIS EXISTS

tracing-test v0.2.6 (crates.io, repo github.com/dbrgn/tracing-test, anti-slopsquat verified live 2026-06-02: max_stable=newest=0.2.6, v0.2.6 created 2026-02-07, NOT yanked) exposes logs_contain(substr) -> bool. It does NOT auto-fail the test. A #[traced_test] body that calls logs_contain WITHOUT wrapping it in assert!/assert_eq! is a silent pass: the macro captures logs, the call returns a bool, the bool is dropped, the test goes green having asserted nothing. This is a Layer-4 fail-open hazard exactly analogous to a wiring gate that emits SKIP on missing evidence. Grounding: wiring-enforcement.md Layer 4 ("in tracing-test #[traced_test] an explicit logs_contain assert is mandatory (the macro does not auto-fail)"); upstream README dbrgn/tracing-test documents logs_contain as a bool-returning helper, not an assertion (research [31]).

THE RULE (blocking)

T-1. Every test function annotated #[traced_test] (or #[tracing_test::traced_test], or via a `use ... as traced_test` alias) MUST contain at least one call to logs_contain or logs_assert that is lexically an argument to assert!, assert_eq!, assert_ne!, debug_assert!, or debug_assert_eq!. A bare `logs_contain("x");` statement, a `let _ = logs_contain(...)`, or a logs_contain inside a non-asserting expression (println!, format!, dbg!, a comparison whose result is dropped) is a FAIL.

T-2. Missing evidence is a hard FAIL (mirrors wiring-enforcement R1): a #[traced_test] with ZERO logs_contain/logs_assert calls is BLOCK, not WARN. The whole point of #[traced_test] is to assert on captured logs; a body that captures and asserts nothing is wire-dark.

T-3. OTel-span variant (forward-looking, same fail-closed posture): if a test asserts on OpenTelemetry spans instead of tracing logs, it MUST use SimpleSpanProcessor + InMemorySpanExporter and assert the exported span set is non-empty AND contains the expected span. BatchSpanProcessor is BANNED in test setup -- its async flush timing makes "not flushed yet" indistinguishable from "never produced", which silently fail-opens the assertion. A test using BatchSpanProcessor in setup is a FAIL.

ENFORCEMENT

The auditor runs .claude/tools/traced_test_assert_check.py over the changed Rust test files (and, on a full audit, over crates/*/tests/ + #[cfg(test)] modules). The checker:
  * finds every fn carrying a #[traced_test] attribute,
  * within that fn body, requires >=1 logs_contain|logs_assert that is lexically an argument to an assert*-family macro,
  * flags BatchSpanProcessor occurrences in any test file,
  * exits nonzero (BLOCK) on any violation; with --expect-traced, a scan that reaches files but finds zero #[traced_test] is itself a FAIL (exit 2) -- the checker self-tests its scan reached the files (the dead-evidence-path screams, R2).
The checker is advisory-unsound (regex + brace-matcher over source, not a full Rust parse) -- it is a Layer-2 static leg, necessary-not-sufficient. The load-bearing backstop is the Layer-4 harness itself (a real assert! that actually fails the test at runtime). This rule guarantees the assert! EXISTS; the harness guarantees it MEANS something.

SELF-CHECK before declaring any #[traced_test] DONE
  1. Does the body call logs_contain or logs_assert?  (if no -> FAIL, T-2)
  2. Is that call an argument to assert!/assert_eq!/assert_ne!?  (if no -> FAIL, T-1)
  3. If it asserts on spans, is it Simple+InMemory (not Batch)?  (if Batch -> FAIL, T-3)
  If any answer is no, the test is fail-open and the verdict is BLOCK.
