//! E-13 — TDD verifier agent driver.
//!
//! `TddVerifierAgent::run_cycle` drives one Red->Green->Refactor
//! cycle for a given `TddTask`. The agent does NOT execute tests
//! itself; it composes the [`super::cycle::RedGreenRefactorState`]
//! state machine with caller-supplied closures that report whether
//! the latest test run flipped from fail to pass.
//!
//! Why the closure-based shape (vs. a full subprocess driver):
//!   The orchestrator already owns a test-runner surface
//!   (`crate::deterministic_auditor`, C-28 mutation runner). The
//!   TDD verifier agent is the controller around those existing
//!   primitives — not a parallel runner. The agent passes the test
//!   result back to the auditor via SSB beacons
//!   (`TddCycleStarted`, `TddTestFailed`, `TddTestPassed`) and
//!   composes the final verdict with D-3 via
//!   [`super::consensus::tdd_results_to_verifier_results`].
//!
//! Iteration cap: a cycle that fails to flip green after
//! `MAX_ITERATIONS` (default 3 — matches the orchestrator's
//! anti-spiral default) returns `CycleResult { test_flipped: false,
//! .. }` so D-3 can fail the verdict cleanly.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::anti_mock::is_mock_free;
use super::cycle::{RedGreenRefactorState, TddCycle};
use super::mutator::MutationOutcome;

/// Maximum number of full Red->Green->Refactor cycles the agent
/// attempts before declaring the task unmet. Anti-spiral bound.
pub const MAX_ITERATIONS: u32 = 3;

/// Input to one TDD cycle drive.
///
/// `task_id` — stable hash-prefix identifier (caller-provided).
/// `failing_test_source` — source of the failing test that opens
///   the Red phase. Used by the anti-mock scanner to verify the
///   test is not mock-heavy.
/// `task_description` — short free-text label for logging /
///   beacons. Not load-bearing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TddTask {
    pub task_id: u64,
    pub task_description: String,
    pub failing_test_source: String,
}

/// Output of one TDD cycle drive.
///
/// `iterations` — number of full Red->Green->Refactor passes the
///   agent executed (0 means the cycle never closed).
/// `test_flipped` — true when at least one Red->Green transition
///   landed within `MAX_ITERATIONS`.
/// `anti_mock_clean` — true when the failing test source did not
///   contain any [`super::anti_mock::MOCK_PATTERNS`] hits.
/// `mutation_outcome` — `Some` only when the caller supplied a
///   mutation probe via `TestRunner::probe_mutants`. The agent does
///   NOT shell out to cargo-mutants on every cycle (cost guard); it
///   leaves that to the caller's discretion.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CycleResult {
    pub iterations: u32,
    pub test_flipped: bool,
    pub anti_mock_clean: bool,
    pub mutation_outcome: Option<MutationOutcome>,
}

/// Closure-based probe the caller passes to `run_cycle` to decide
/// whether the latest test run flipped green. The agent is
/// runtime-agnostic — the caller decides whether the probe shells
/// out to `cargo test`, calls into an in-process test harness, or
/// uses a recorded fixture (for the contract tests).
pub type TestProbe<'a> = &'a mut dyn FnMut(TddCycle) -> bool;

/// Errors that surface during a TDD cycle drive.
#[derive(Debug, Error)]
pub enum TddVerifierError {
    /// The task's `failing_test_source` is empty. The Red phase
    /// requires at least a non-blank test to start the cycle.
    #[error("TDD task has an empty failing_test_source")]
    EmptyFailingTestSource,
    /// The iteration cap was reached without the test ever
    /// flipping to green. The caller still receives a
    /// `CycleResult` with `test_flipped: false` so D-3 can emit the
    /// fail verdict; this error variant is reserved for the
    /// (rarer) case where the agent could not even attempt a
    /// cycle.
    #[error("TDD cycle exhausted iteration cap ({0}) without flipping green")]
    IterationCapExhausted(u32),
}

/// Driver for one TDD verification pass.
///
/// `TddVerifierAgent` is a thin orchestration shell — it holds the
/// in-progress [`RedGreenRefactorState`] and runs the cycle to
/// closure. State is plain data + serde-derivable so SSB beacons
/// can carry the agent's view at any point.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TddVerifierAgent {
    pub state: RedGreenRefactorState,
}

impl TddVerifierAgent {
    /// Construct a fresh agent at the Red phase.
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: RedGreenRefactorState::new(),
        }
    }

    /// Drive one TDD cycle to closure (or to the iteration cap).
    ///
    /// `probe` is called once per phase; the agent advances the
    /// internal state machine and records whether the test passed
    /// at the Green and Refactor checkpoints.
    ///
    /// The function does NOT emit SSB rows itself — the caller is
    /// expected to inspect the returned `CycleResult` and emit the
    /// appropriate beacons via the orchestrator's SSB producer
    /// chain. This separation keeps the agent unit-testable.
    pub fn run_cycle(
        &mut self,
        task: &TddTask,
        probe: TestProbe<'_>,
    ) -> Result<CycleResult, TddVerifierError> {
        if task.failing_test_source.trim().is_empty() {
            return Err(TddVerifierError::EmptyFailingTestSource);
        }

        let anti_mock_clean = is_mock_free(&task.failing_test_source);
        let mut flipped = false;

        for _ in 0..MAX_ITERATIONS {
            // Red phase: test should fail. If it already passes,
            // the cycle wasn't really Red — caller error, but we
            // still advance.
            let red_pass = probe(TddCycle::Red);
            self.state.record_test(red_pass);

            self.state.advance(); // Red -> Green

            // Green phase: caller should have applied the production
            // change; the test must now pass.
            let green_pass = probe(TddCycle::Green);
            self.state.record_test(green_pass);
            if green_pass {
                flipped = true;
            }

            self.state.advance(); // Green -> Refactor

            // Refactor phase: test must STAY green.
            let refactor_pass = probe(TddCycle::Refactor);
            self.state.record_test(refactor_pass);

            self.state.advance(); // Refactor -> Red (iterations += 1)

            if flipped && refactor_pass {
                break;
            }
        }

        Ok(CycleResult {
            iterations: self.state.iterations,
            test_flipped: flipped,
            anti_mock_clean,
            mutation_outcome: None,
        })
    }
}

impl Default for TddVerifierAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_task(src: &str) -> TddTask {
        TddTask {
            task_id: 42,
            task_description: "test task".into(),
            failing_test_source: src.into(),
        }
    }

    #[test]
    fn empty_failing_test_source_errors() {
        let mut agent = TddVerifierAgent::new();
        let task = mk_task("   \n\t  ");
        let mut probe = |_: TddCycle| true;
        match agent.run_cycle(&task, &mut probe) {
            Err(TddVerifierError::EmptyFailingTestSource) => (),
            other => panic!("expected EmptyFailingTestSource, got {other:?}"),
        }
    }

    #[test]
    fn happy_path_flips_green_first_iteration() {
        let mut agent = TddVerifierAgent::new();
        let task = mk_task("#[test] fn foo() { assert_eq!(2 + 2, 4); }");
        // Probe: Red fails, Green passes, Refactor passes.
        let mut step = 0;
        let mut probe = |phase: TddCycle| {
            step += 1;
            match phase {
                TddCycle::Red => false,
                TddCycle::Green | TddCycle::Refactor => true,
            }
        };
        let result = agent.run_cycle(&task, &mut probe).expect("cycle runs");
        assert!(result.test_flipped);
        assert!(result.anti_mock_clean);
        assert_eq!(result.iterations, 1);
    }

    #[test]
    fn fails_after_max_iterations_returns_no_flip() {
        let mut agent = TddVerifierAgent::new();
        let task = mk_task("#[test] fn foo() {}");
        // Probe always fails — flip never happens.
        let mut probe = |_: TddCycle| false;
        let result = agent.run_cycle(&task, &mut probe).expect("cycle runs");
        assert!(!result.test_flipped);
        assert_eq!(result.iterations, MAX_ITERATIONS);
    }

    #[test]
    fn anti_mock_dirty_test_source_reported() {
        let mut agent = TddVerifierAgent::new();
        let task = mk_task("#[mockall] fn foo() {}");
        let mut probe = |phase: TddCycle| matches!(phase, TddCycle::Green | TddCycle::Refactor);
        let result = agent.run_cycle(&task, &mut probe).expect("cycle runs");
        assert!(!result.anti_mock_clean);
    }
}
