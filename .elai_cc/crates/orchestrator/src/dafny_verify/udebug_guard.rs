//! C-52 — uDebug-style vacuous-verification guard per arXiv:2604.22601.
//!
//! Paper §3 documents `vacuous_verification` as a failure mode: a
//! Dafny spec that "verifies" trivially (e.g. constant return,
//! empty postcondition) passes the verifier yet fails simple
//! functional tests. Raw `dafny verify` exit-zero is INSUFFICIENT —
//! the spec MUST ALSO pass a uDebug-style functional test suite to
//! be considered verified.
//!
//! This module ships the host-side guard. Functional-test execution
//! is the caller's responsibility (e.g. translating the paper's
//! UVa Online Judge ground-truth tests into Rust assertions or
//! shelling out to a Python harness). The guard only computes the
//! GO / NO-GO decision from the test results.

use serde::{Deserialize, Serialize};

/// Per-test outcome for a single functional test in the uDebug-style
/// suite. The contract test G3 asserts that an empty test vector OR
/// any failing test flips `check_vacuous` to `true`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FunctionalTestResult {
    /// Human-readable test name (UVa problem id + case index per
    /// paper convention).
    pub name: String,
    /// Whether the functional test passed when run against the
    /// emitted Dafny spec / extracted code.
    pub passed: bool,
}

impl FunctionalTestResult {
    /// Convenience constructor used in tests + CLI fixtures.
    pub fn new(name: impl Into<String>, passed: bool) -> Self {
        Self {
            name: name.into(),
            passed,
        }
    }
}

/// Decide whether a spec is VACUOUS (paper §3 vacuous-verification
/// failure mode).
///
/// Returns `true` (VACUOUS, REJECT) iff:
///   - `functional_tests` is empty (no evidence of non-vacuous
///     coverage — empty suites are treated as inadequate
///     verification), OR
///   - ANY functional test FAILED (the spec verifies in Dafny but a
///     concrete behavioural check disagreed).
///
/// Returns `false` (NON-VACUOUS, ACCEPT) only when every functional
/// test passed AND there was at least one functional test to begin
/// with.
///
/// Note: the `spec` argument is currently used only as a stability
/// anchor (the guard's caller may inspect it to log a sample of the
/// rejected spec). It is NOT parsed; only the test-result vector
/// drives the decision. The argument is kept in the signature so
/// future expansion (e.g. AST-based "empty body" heuristic) is
/// non-breaking.
pub fn check_vacuous(_spec: &str, functional_tests: &[FunctionalTestResult]) -> bool {
    if functional_tests.is_empty() {
        // No functional coverage = inadequate verification = vacuous.
        return true;
    }
    functional_tests.iter().any(|t| !t.passed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_test_suite_is_vacuous() {
        let spec = "method foo() returns (x: int) { x := 0; }";
        let tests: Vec<FunctionalTestResult> = Vec::new();
        assert!(
            check_vacuous(spec, &tests),
            "empty functional-test suite must be flagged vacuous"
        );
    }

    #[test]
    fn any_failing_test_is_vacuous() {
        let spec = "method foo() returns (x: int) { x := 0; }";
        let tests = vec![
            FunctionalTestResult::new("uva-100-case-1", true),
            FunctionalTestResult::new("uva-100-case-2", false),
        ];
        assert!(
            check_vacuous(spec, &tests),
            "any failing functional test must be flagged vacuous"
        );
    }

    #[test]
    fn all_passing_tests_are_not_vacuous() {
        let spec = "method foo() returns (x: int) { x := 0; }";
        let tests = vec![
            FunctionalTestResult::new("uva-100-case-1", true),
            FunctionalTestResult::new("uva-100-case-2", true),
        ];
        assert!(
            !check_vacuous(spec, &tests),
            "every functional test passing means non-vacuous"
        );
    }
}
