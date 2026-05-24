//! E-13 — TDD Red/Green/Refactor cycle types.
//!
//! The cycle is a closed three-arm enum matching the classic
//! Beck-style TDD discipline (write a failing test = Red, make it
//! pass with the minimum code = Green, refactor without breaking the
//! green bar = Refactor). The state machine is intentionally simple:
//! callers transition Red -> Green -> Refactor and either close the
//! cycle (`is_closed()`) or loop back to Red for the next task.
//!
//! The plan's larger `RedGreenRefactorState` surface was simplified
//! per the re-check report's scope-lean lean: the state struct
//! carries the current phase + the last observed test outcome
//! (`last_test_passed`) so the agent driver can report
//! `TddTestPassed` / `TddTestFailed` beacons without re-running the
//! test in the next call.

use serde::{Deserialize, Serialize};

/// Closed three-arm enum for the canonical TDD cycle phases.
///
/// `Red` -- a failing test exists for the task under development;
/// no production code has been written yet (or the production code
/// fails the new test).
/// `Green` -- the production code now passes the previously-Red
/// test. The test outcome flipped from fail to pass within this
/// cycle.
/// `Refactor` -- the production code is being cleaned up without
/// changing observable behavior. Tests MUST stay green throughout
/// this phase; the auditor blocks if a previously-green test goes
/// red during refactor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TddCycle {
    Red,
    Green,
    Refactor,
}

impl TddCycle {
    /// Snake-case wire label used in SSB rows + dashboard rendering.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Red => "red",
            Self::Green => "green",
            Self::Refactor => "refactor",
        }
    }

    /// Compute the next phase in the canonical cycle. The cycle
    /// always advances forward (no skip-Green-from-Red); the agent
    /// closes the cycle when it returns to `Red` for the next task.
    #[must_use]
    pub fn next(self) -> Self {
        match self {
            Self::Red => Self::Green,
            Self::Green => Self::Refactor,
            Self::Refactor => Self::Red,
        }
    }
}

impl std::fmt::Display for TddCycle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Persistent state of a single TDD cycle drive. Held by the
/// [`super::TddVerifierAgent`] across phase transitions.
///
/// `phase` -- current `TddCycle` variant.
/// `last_test_passed` -- outcome of the most recent test run. `None`
/// when the agent has not yet executed a test (initial state before
/// the first Red->Green transition).
/// `iterations` -- how many full Red->Green->Refactor cycles have
/// completed for this task. Bounded by the agent's iteration cap.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedGreenRefactorState {
    pub phase: TddCycle,
    pub last_test_passed: Option<bool>,
    pub iterations: u32,
}

impl RedGreenRefactorState {
    /// Construct a fresh cycle state at the Red phase with no test
    /// history.
    #[must_use]
    pub fn new() -> Self {
        Self {
            phase: TddCycle::Red,
            last_test_passed: None,
            iterations: 0,
        }
    }

    /// Advance the cycle one phase forward. Increments `iterations`
    /// when the cycle wraps from Refactor back to Red.
    pub fn advance(&mut self) {
        let next = self.phase.next();
        if next == TddCycle::Red {
            self.iterations = self.iterations.saturating_add(1);
        }
        self.phase = next;
    }

    /// Record the result of the most recent test run. Idempotent.
    pub fn record_test(&mut self, passed: bool) {
        self.last_test_passed = Some(passed);
    }

    /// True once the cycle has executed at least one full
    /// Red->Green->Refactor transition AND the last observed test
    /// outcome is `Some(true)`. Used by the agent driver to decide
    /// when to emit `TddTestPassed`.
    #[must_use]
    pub fn is_closed(&self) -> bool {
        self.iterations > 0 && self.last_test_passed == Some(true)
    }
}

impl Default for RedGreenRefactorState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycle_next_wraps_refactor_to_red() {
        assert_eq!(TddCycle::Red.next(), TddCycle::Green);
        assert_eq!(TddCycle::Green.next(), TddCycle::Refactor);
        assert_eq!(TddCycle::Refactor.next(), TddCycle::Red);
    }

    #[test]
    fn cycle_as_str_snake_case_round_trip() {
        let labels = ["red", "green", "refactor"];
        let phases = [TddCycle::Red, TddCycle::Green, TddCycle::Refactor];
        for (phase, label) in phases.iter().zip(labels.iter()) {
            assert_eq!(phase.as_str(), *label);
            assert_eq!(format!("{phase}"), *label);
        }
    }

    #[test]
    fn state_advance_increments_iterations_on_red_wrap() {
        let mut state = RedGreenRefactorState::new();
        assert_eq!(state.iterations, 0);
        state.advance(); // Red -> Green
        assert_eq!(state.iterations, 0);
        state.advance(); // Green -> Refactor
        assert_eq!(state.iterations, 0);
        state.advance(); // Refactor -> Red (cycle wraps)
        assert_eq!(state.iterations, 1);
    }

    #[test]
    fn state_is_closed_requires_iteration_and_pass() {
        let mut state = RedGreenRefactorState::new();
        state.record_test(true);
        // No completed iteration yet.
        assert!(!state.is_closed());
        for _ in 0..3 {
            state.advance();
        }
        state.record_test(true);
        assert!(state.is_closed());
        state.record_test(false);
        assert!(!state.is_closed());
    }
}
