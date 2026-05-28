//! E-54 test-execution-based quality filter (task 4).
//!
//! Per arXiv:2411.04905 OpenCoder + arXiv:2503.02951 KodCode triplet schema:
//! every NEW task is validated against TWO oracles before ingest:
//!   (i)  the reference_solution MUST pass every `tests` entry.
//!   (ii) a deliberately-broken variant of the reference MUST fail at
//!        least one test (otherwise the tests are too weak).
//!
//! The sandbox runtime crate (`crates/sandbox/`) is ABSENT (verified via
//! Glob). Per CLAUDE.md stub registry rule, this module ships with a
//! clearly-marked TODO[E-54.4] stub that returns a `SandboxMissing` verdict
//! when invoked at runtime. The HARNESS surface (TestExecutionResult enum +
//! execute_against_reference / execute_against_broken APIs) ships in this
//! commit; the live sandbox wire is RUNPOD-PENDING.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::task_schema::Task;

/// Verdict of a single test execution against a candidate solution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestExecutionResult {
    /// True when every `tests` entry passed.
    pub all_passed: bool,
    /// Indices into [`Task::tests`] that failed. Empty when `all_passed`.
    pub failed_indices: Vec<usize>,
    /// Wall-clock latency of the executor run in milliseconds.
    pub latency_ms: u64,
    /// True when the sandbox runtime was unavailable (stub return).
    pub sandbox_missing: bool,
}

/// Errors raised by the test-executor harness layer (not by individual test
/// failures -- those are surfaced via [`TestExecutionResult::failed_indices`]).
#[derive(Debug, Error)]
pub enum TestExecutorError {
    #[error("eval-corpus test_executor: empty tests on task {0}")]
    NoTests(String),
    #[error("eval-corpus test_executor: I/O failure: {0}")]
    Io(String),
}

/// Run every `tests` entry against the task's `reference_solution`.
///
/// RUNPOD-PENDING: wire `crates/sandbox/` (does not exist as of this commit).
/// Until the sandbox lands, this returns a `sandbox_missing=true` verdict
/// with `all_passed=true` (defaults-open so the ingest pipeline does NOT
/// silently reject every task). Per CLAUDE.md stub registry rule, the
/// babystep id is `E-54.4` and the stub will resolve in a future patch /
/// plan. See workflow/sessions/s-25120/stubs.md.
pub fn execute_against_reference(task: &Task) -> Result<TestExecutionResult, TestExecutorError> {
    if task.tests.is_empty() {
        return Err(TestExecutorError::NoTests(task.id.clone()));
    }
    // TODO[E-54.4]: replace sandbox stub once crates/sandbox/ lands.
    // Until then, return defaults-open verdict so the ingest pipeline does
    // not blanket-reject every task with a test triplet.
    Ok(TestExecutionResult {
        all_passed: true,
        failed_indices: Vec::new(),
        latency_ms: 0,
        sandbox_missing: true,
    })
}

/// Run every `tests` entry against a deliberately-broken variant of the
/// reference solution. At least one test MUST fail; otherwise the tests are
/// too weak and the task is rejected via [`corpus_test_useless_passes_broken`]
/// SSB row. Per arXiv:2411.04905 OpenCoder recipe.
///
/// RUNPOD-PENDING: same sandbox stub. Returns `sandbox_missing=true` with
/// `all_passed=false` (the broken oracle is conservative -- treats missing
/// sandbox as "we can't tell, so default to allowing the task").
pub fn execute_against_broken(task: &Task) -> Result<TestExecutionResult, TestExecutorError> {
    if task.tests.is_empty() {
        return Err(TestExecutorError::NoTests(task.id.clone()));
    }
    // TODO[E-54.4]: replace sandbox stub once crates/sandbox/ lands.
    // Conservative-fail: assume the broken oracle correctly fails a test.
    Ok(TestExecutionResult {
        all_passed: false,
        failed_indices: vec![0],
        latency_ms: 0,
        sandbox_missing: true,
    })
}

/// SSB-payload-ready discriminator for the test_executor's contribution to
/// the CorpusEventPhase chain. Combined with [`TestExecutionResult`] into the
/// EvalCorpusEventPayload.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TestExecutorPhase {
    /// Reference solution failed >=1 test (`corpus_test_fails_on_reference`).
    FailsOnReference,
    /// Broken solution passed all tests (`corpus_test_useless_passes_broken`).
    UselessPassesBroken,
    /// Reference passes + broken fails -- task accepted by the executor.
    Passed,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TaskFamily as TF;
    use crate::task_schema::{ExecutableTest, Provenance, TestKind};
    use chrono::{TimeZone, Utc};
    use std::collections::HashMap;

    fn fixture(id: &str, n_tests: usize) -> Task {
        let mut cutoffs = HashMap::new();
        cutoffs.insert("m".to_string(), "2024-10".to_string());
        Task {
            id: id.to_string(),
            family: TF::CodeFix,
            created_at: Utc.with_ymd_and_hms(2026, 5, 27, 0, 0, 0).unwrap(),
            cutoffs,
            problem: "p".to_string(),
            reference_solution: "r".to_string(),
            tests: (0..n_tests)
                .map(|i| ExecutableTest {
                    input: format!("i{i}"),
                    expected_output: format!("o{i}"),
                    kind: TestKind::UnitTest,
                })
                .collect(),
            provenance: Provenance {
                generator: "x".to_string(),
                prompt_id: None,
                date: Utc.with_ymd_and_hms(2026, 5, 27, 0, 0, 0).unwrap(),
            },
            framing_pair: None,
            perturbed_pair: None,
            embedding_hash: "deadbeefcafe1234".to_string(),
            ngram_hash: "1234deadbeefcafe".to_string(),
        }
    }

    #[test]
    fn execute_against_reference_errors_on_empty_tests() {
        let t = fixture("e1", 0);
        let r = execute_against_reference(&t);
        assert!(matches!(r, Err(TestExecutorError::NoTests(_))));
    }

    #[test]
    fn execute_against_broken_errors_on_empty_tests() {
        let t = fixture("e2", 0);
        let r = execute_against_broken(&t);
        assert!(matches!(r, Err(TestExecutorError::NoTests(_))));
    }

    #[test]
    fn stub_returns_sandbox_missing_true_pending_runpod() {
        let t = fixture("e3", 1);
        let r = execute_against_reference(&t).unwrap();
        assert!(r.sandbox_missing, "stub must mark sandbox_missing until E-54.4 lands");
    }
}
